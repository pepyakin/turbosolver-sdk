package me.pepyakin.turbosolver.capnp

import android.util.SparseArray
import io.reactivex.Completable
import io.reactivex.Single
import io.reactivex.SingleEmitter
import me.pepyakin.turbosolver.LocalHttpTurboSolverFactory
import me.pepyakin.turbosolver.TurboSolver
import me.pepyakin.turbosolver.TurboSolverFactory
import org.capnproto.MessageBuilder
import org.capnproto.Serialize
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.channels.Channels
import java.util.concurrent.atomic.AtomicInteger

class CapnpTurboSolver constructor(
        private val id: Int,
        private val dispatcher: Dispatcher): TurboSolver {

    override fun solve(): Single<String> {
        return dispatcher
                .dispatch {
                    initSolveReq().apply {
                        id = this@CapnpTurboSolver.id
                    }
                }.map { resp ->
                    when (resp) {
                        is Resp.SolveResult ->  resp.solution
                        else -> error("unexpected variant! $resp")
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
                .doOnSuccess {
                    assert(it is Resp.SolverDestroyed)
                }
                .toCompletable()
    }
}

class CapnpTurboSolverFactory private constructor(
        private val dispatcher: Dispatcher): TurboSolverFactory {

    companion object {
        fun create(): CapnpTurboSolverFactory =
                CapnpTurboSolverFactory(Dispatcher())
    }

    override fun create(grid: String): Single<TurboSolver> {
        return dispatcher
                .dispatch {
                    initCreateSolverReq().setGrid(grid)
                }
                .map { resp ->
                    when (resp) {
                        is Resp.SolverCreated ->
                            CapnpTurboSolver(resp.id, dispatcher)
                        else -> error("unexpected variant! $resp")
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

    private val dispatcherPtr: Long
    init {
        dispatcherPtr = capnp_init(this)
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

            // Send the bytes array to a backend.
            capnp_send(dispatcherPtr, bytes)

            synchronized(emitterStash) {
                emitterStash.put(reqId, emitter)
            }
        }
    }

    // Called from JNI.
    @Suppress("unused")
    fun callback(data: ByteBuffer) {
        val respReader = Serialize.read(data).getRoot(Api.Resp.factory)
        val resp = Resp.fromReader(respReader)

        // Get emitter out under lock.
        val emitter = synchronized(emitterStash) {
            val e = emitterStash[respReader.id]
            emitterStash.remove(respReader.id)
            e
        }

        // Emit successful response.
        emitter.onSuccess(resp)
    }
}

sealed class Resp {
    companion object {
        fun fromReader(reader: Api.Resp.Reader): Resp = when (reader.which()!!) {
            Api.Resp.Which.CREATE_SOLVER_RESP ->
                Resp.SolverCreated(reader.createSolverResp.id)

            Api.Resp.Which.SOLVE_RESP ->
                Resp.SolveResult(reader.solveResp.solution.toString())

            Api.Resp.Which.DESTROY_RESP ->
                Resp.SolverDestroyed

            Api.Resp.Which._NOT_IN_SCHEMA -> error("variant not in the schema")
        }
    }

    data class SolverCreated(val id: Int): Resp()
    data class SolveResult(val solution: String): Resp()
    object SolverDestroyed: Resp()
}
