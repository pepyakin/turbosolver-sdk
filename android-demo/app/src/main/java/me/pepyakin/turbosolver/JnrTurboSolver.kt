package me.pepyakin.turbosolver

import android.content.Context
import io.reactivex.Completable
import io.reactivex.Single
import jnr.ffi.LibraryLoader
import jnr.ffi.Pointer
import jnr.ffi.annotations.Delegate
import java.io.File
import java.io.InputStream
import java.io.OutputStream
import java.util.concurrent.atomic.AtomicBoolean

class JnrTurboSolverException(msg: String): RuntimeException(msg)

interface NativeTurboSolverApi {
    fun solver_create(grid: CharSequence, cb: SolverCreatedCallback)
    fun solver_solve(solver_ptr: Pointer, cb: SolverResultCallback)
    fun solver_destroy(solver_ptr: Pointer)
}

interface SolverCreatedCallback {
    @Delegate
    fun onSolverCreated(solver_ptr: Pointer?, error: CharSequence?)
}

interface SolverResultCallback {
    @Delegate
    fun onSolverResult(solution: CharSequence?)
}

class JnrTurboSolver(
        private val nativeSolver: NativeTurboSolverApi,
        private val solverPtr: Pointer): TurboSolver {

    private val destroyed = AtomicBoolean(false)

    override fun solve(): Single<String> {
        return Single.create<String> { emitter ->
            if (destroyed.get()) {
                throw JnrTurboSolverException("solver is destroyed")
            }
            nativeSolver.solver_solve(solverPtr, object: SolverResultCallback {
                override fun onSolverResult(solution: CharSequence?) {
                    if (emitter.isDisposed) {
                        return
                    }

                    if (solution != null) {
                        val sb = StringBuilder()
                        sb.append(solution)
                        emitter.onSuccess(sb.toString())
                    } else {
                        emitter.onError(JnrTurboSolverException("solution not found"))
                    }
                }
            })
        }
    }

    override fun destroy(): Completable {
        return Completable.create { emitter ->
            if (!destroyed.compareAndSet(false, true)) {
                throw JnrTurboSolverException("solver is already destroyed")
            }
            nativeSolver.solver_destroy(solverPtr)
            emitter.onComplete()
        }
    }
}

/**
 * Initialize JNR and load libsolver.so.
 *
 * It might take some time! Make sure you calling this from background thread.
 */
private fun initNativeSolverApi(context: Context): NativeTurboSolverApi {
    // JNR expects that this property always present. Any value other than 32 or 64 will
    // trigger auto-detection of data model.
    System.setProperty("sun.arch.data.model", "0")

    // JNR tries to load libraries from JAR at jni/**/*.so. Let alone that loading
    // files from JAR is TERRIBLY SLOW, it doesn't work, because gradle strips all *.so
    // files from JARs, and there are no way to prevent this (packagingOptions doesn't work).
    // If you want to dig deeper, you can start at JarMerger.addJar in Gradle builder.
    // Also, related https://github.com/jnr/jffi/issues/17
    //
    // So, to workaround that we what we would do is copy all jffi-native libs into the local
    // folder and override library path.
    val filesDir = context.filesDir
    val jffiNativesDir = File(filesDir, "jffi-native")
    val x86_64Linux = File(jffiNativesDir, "x86_64-Linux")
    x86_64Linux.mkdirs()
    System.setProperty("jffi.boot.library.path", jffiNativesDir.canonicalPath)

    val libjffi = File(x86_64Linux, "libjffi-1.2.so")
    if (!libjffi.exists()) {
        // Do the actual copy. I can afford to copy the only supported x86_64 so :p
        val input = context.resources.assets.open("jffi-native/x86_64-Linux/libjffi-1.2.so")
        val output = libjffi.outputStream()

        copy(input, output)
    }

    return LibraryLoader.create(NativeTurboSolverApi::class.java).load("solver")
}

private fun copy(input: InputStream, output: OutputStream) {
    val buffer = ByteArray(1024)
    while (true) {
        val bytesRead = input.read(buffer)
        if (bytesRead == -1) break
        output.write(buffer, 0, bytesRead)
    }

    input.close()
    output.close()
}

class JnrTurboSolverFactory private constructor(
        val nativeSolver: NativeTurboSolverApi): TurboSolverFactory {

    companion object {
        private var nativeSolver: NativeTurboSolverApi? = null

        fun create(context: Context): JnrTurboSolverFactory {
            synchronized(this) {
                if (nativeSolver == null) {
                    nativeSolver = initNativeSolverApi(context.applicationContext)
                }
            }
            return JnrTurboSolverFactory(nativeSolver!!)
        }
    }

    override fun create(grid: String): Single<TurboSolver> {
        return Single.create<TurboSolver> { emitter ->
            nativeSolver.solver_create(grid, object: SolverCreatedCallback {
                override fun onSolverCreated(solver_ptr: Pointer?, error: CharSequence?) {
                    if (emitter.isDisposed) {
                        return
                    }

                    if (error != null) {
                        // Note that we don't use CharSequence.toString here because there is no
                        // guarantee it will return string from chars that it contains.
                        val sb = StringBuilder()
                        sb.append(error)
                        val exception = JnrTurboSolverException(sb.toString())

                        emitter.onError(exception)
                        return
                    }

                    if (solver_ptr != null) {
                        emitter.onSuccess(JnrTurboSolver(nativeSolver, solver_ptr))
                    } else {
                        kotlin.error("error == null and solver_ptr == null?")
                    }
                }
            })
        }
    }
}
