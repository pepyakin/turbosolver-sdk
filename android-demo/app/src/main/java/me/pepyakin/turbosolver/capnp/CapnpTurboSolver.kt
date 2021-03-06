package me.pepyakin.turbosolver.capnp

import android.util.SparseArray
import io.reactivex.Completable
import io.reactivex.Single
import io.reactivex.SingleEmitter
import me.pepyakin.turbosolver.TurboSolver
import me.pepyakin.turbosolver.TurboSolverFactory
import org.capnproto.MessageBuilder
import org.capnproto.Serialize
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.channels.Channels
import java.util.concurrent.atomic.AtomicInteger

class CapnpTurboSolverException(errno: Int): Exception("err: $errno")

class CapnpTurboSolver constructor(
        private val id: Int,
        private val dispatcher: Dispatcher): TurboSolver {

    override fun solve(): Single<String> {
        return dispatcher
                .dispatch {
                    initSolveReq().apply {
                        id = this@CapnpTurboSolver.id
                    }
                }
                .extractKind()
                .map {
                    when (it) {
                        is RespKind.SolveResult -> it.solution
                        else -> error("unexpected variant! $it")
                    }
                }
    }

    override fun destroy(): Completable {
        return dispatcher
                .dispatch {
                    initDestroyReq().apply {
                        id = this@CapnpTurboSolver.id
                    }
                }
                .extractKind()
                .doOnSuccess {
                    assert(it is RespKind.SolverDestroyed)
                }
                .toCompletable()
    }
}

class CapnpTurboSolverFactory private constructor(
        private val dispatcher: Dispatcher): TurboSolverFactory {

    companion object {
        @JvmStatic
        fun create(): CapnpTurboSolverFactory =
                CapnpTurboSolverFactory(Dispatcher())
    }

    override fun create(grid: String): Single<TurboSolver> {
        return dispatcher
                .dispatch {
                    initCreateSolverReq().setGrid(grid)
                }
                .extractKind()
                .map {
                    when (it) {
                        is RespKind.SolverCreated ->
                            CapnpTurboSolver(it.id, dispatcher)
                        else -> error("unexpected variant! $it")
                    }
                }
    }
}

class Dispatcher {
    companion object {
        @JvmStatic
        external fun capnp_init(self: Dispatcher): Long

        @JvmStatic
        external fun capnp_send(dispatcher: Long, data: ByteArray)

        init {
            System.loadLibrary("solver")
        }
    }

    private val executorPtr: Long
    init {
        executorPtr = capnp_init(this)
    }

    private var nextId = AtomicInteger()
    private val emitterStash = SparseArray<SingleEmitter<Resp>>()

    private fun generateId(): Int = nextId.getAndIncrement()

    fun dispatch(buildMsg: Api.Req.Builder.() -> Unit): Single<Resp> {
        val reqId = generateId()
        return Single.create<Resp> { emitter ->
            val message = MessageBuilder()
            message.initRoot(Api.Req.factory).apply {
                id = reqId
                buildMsg()
            }

            // Serialize message into byte array
            val baos = ByteArrayOutputStream()
            val out = Channels.newChannel(baos)
            Serialize.write(out, message)
            val bytes = baos.toByteArray()

            // Put emitter onto stash sending msg. It is important to
            // do it before sending message to the backend.
            synchronized(emitterStash) {
                emitterStash.put(reqId, emitter)
            }

            // Send the bytes array to the backend.
            capnp_send(executorPtr, bytes)
        }
    }

    // Called from JNI.
    @Suppress("unused")
    fun callback(data: ByteBuffer) {
        val respReader = Serialize.read(data).getRoot(Api.Resp.factory)
        val resp = Resp.fromReader(respReader)

        // Get emitter out under the lock.
        val emitter = synchronized(emitterStash) {
            val e = emitterStash[respReader.id]
            emitterStash.remove(respReader.id)
            e
        }

        // Emit successful response.
        emitter.onSuccess(resp)
    }
}

fun Single<Resp>.extractKind(): Single<RespKind> = flatMap { resp ->
    when (resp) {
        is Resp.Ok -> Single.just(resp.respKind)
        is Resp.Errno -> Single.error(CapnpTurboSolverException(resp.errno))
    }
}

sealed class Resp {
    companion object {
        fun fromReader(reader: Api.Resp.Reader): Resp = when (reader.which()!!) {
            Api.Resp.Which.OK ->
                Resp.Ok(RespKind.fromReader(reader.ok))

            Api.Resp.Which.ERR->
                Resp.Errno(reader.err)

            Api.Resp.Which._NOT_IN_SCHEMA -> error("variant not in the schema")
        }
    }

    data class Ok(val respKind: RespKind): Resp()
    data class Errno(val errno: Int): Resp()
}

sealed class RespKind {
    companion object {
        fun fromReader(reader: Api.OkResp.Reader): RespKind = when (reader.which()!!) {
            Api.OkResp.Which.CREATE_SOLVER_RESP ->
                RespKind.SolverCreated(reader.createSolverResp.id)

            Api.OkResp.Which.SOLVE_RESP ->
                RespKind.SolveResult(reader.solveResp.solution.toString())

            Api.OkResp.Which.DESTROY_RESP ->
                RespKind.SolverDestroyed

            Api.OkResp.Which._NOT_IN_SCHEMA -> error("variant not in the schema")
        }
    }

    data class SolverCreated(val id: Int): RespKind()
    data class SolveResult(val solution: String): RespKind()
    object SolverDestroyed: RespKind()
}
