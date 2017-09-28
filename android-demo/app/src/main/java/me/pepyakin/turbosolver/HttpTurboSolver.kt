
package me.pepyakin.turbosolver

import io.reactivex.Completable
import io.reactivex.Observable
import io.reactivex.Single
import io.reactivex.schedulers.Schedulers
import retrofit2.Retrofit
import retrofit2.adapter.rxjava2.RxJava2CallAdapterFactory
import retrofit2.converter.moshi.MoshiConverterFactory
import retrofit2.http.*

data class CreateSolverReq(val grid: String)
data class CreateSolverResp(val id: Int)
data class SolutionResp(val solution: String)

/**
 * HTTP API for the local TurboSolver server.
 */
interface LocalTurboSolverApi {
    @POST("/")
    fun create(@Body req: CreateSolverReq): Observable<CreateSolverResp>

    @GET("/{id}/solution")
    fun solution(@Path("id") id: Int): Observable<SolutionResp>

    @DELETE("/{id}")
    fun destroy(@Path("id") id: Int): Completable
}

class LocalHttpTurboSolver(
        private val id: Int,
        private val localTurboSolverApi: LocalTurboSolverApi
) : TurboSolver {
    override fun solve(): Single<String> =
            localTurboSolverApi.solution(id)
                    .singleOrError()
                    .map { it.solution }
                    .subscribeOn(Schedulers.io())

    override fun destroy(): Completable =
            localTurboSolverApi.destroy(id)
                    .subscribeOn(Schedulers.io())
}

class LocalHttpTurboSolverFactory(
        private val localTurboSolverApi: LocalTurboSolverApi
) : TurboSolverFactory {
    companion object {
        @JvmStatic
        private external fun deploy()

        init {
            System.loadLibrary("solver")
            Thread({
                deploy()
            }).start()
        }

        @JvmStatic
        fun create(port: Int = 8000): LocalHttpTurboSolverFactory {
            val retrofit = Retrofit.Builder()
                    .baseUrl("http://localhost:$port")
                    .addCallAdapterFactory(RxJava2CallAdapterFactory.create())
                    .addConverterFactory(MoshiConverterFactory.create())
                    .build()
            val solverApi = retrofit.create(LocalTurboSolverApi::class.java)

            return LocalHttpTurboSolverFactory(solverApi)
        }
    }

    override fun create(grid: String): Single<TurboSolver> {
        val req = CreateSolverReq(grid)
        return localTurboSolverApi
                .create(req)
                .singleOrError()
                .subscribeOn(Schedulers.io())
                .map {
                    LocalHttpTurboSolver(
                            id = it.id,
                            localTurboSolverApi = localTurboSolverApi
                    )
                }
    }
}
