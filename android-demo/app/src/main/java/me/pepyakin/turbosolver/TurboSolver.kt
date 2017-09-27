package me.pepyakin.turbosolver

import io.reactivex.Observable
import io.reactivex.Single
import retrofit2.Retrofit
import retrofit2.adapter.rxjava2.RxJava2CallAdapterFactory
import retrofit2.converter.moshi.MoshiConverterFactory
import retrofit2.http.*

interface TurboSolver {
    fun solve(): Single<String>
    fun destroy(): Single<Unit>
}

interface TurboSolverFactory {
    fun create(grid: String): Single<TurboSolver>
}

data class CreateSolverReq(val grid: String)
data class CreateSolverResp(val id: Int)

interface LocalTurboSolverApi {
    @POST("")
    fun create(@Body req: CreateSolverReq): Observable<CreateSolverResp>

    @GET("{id}/solution")
    fun solution(@Path("id") id: Int): Observable<String>

    @DELETE("{id}")
    fun destroy(@Path("id") id: Int): Observable<Unit>
}

class LocalHttpTurboSolver(
        private val id: Int,
        private val localTurboSolverApi: LocalTurboSolverApi
) : TurboSolver {
    override fun solve(): Single<String> =
            localTurboSolverApi.solution(id).singleOrError()

    override fun destroy(): Single<Unit> =
            localTurboSolverApi.destroy(id).singleOrError()
}

class LocalHttpTurboSolverFactory(
        private val localTurboSolverApi: LocalTurboSolverApi
) : TurboSolverFactory {
    companion object {
        @JvmStatic
        external fun deploy()

        init {
            System.loadLibrary("solver")
            deploy()
        }

        @JvmStatic
        fun create(port: Int): LocalHttpTurboSolverFactory {
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
                .map {
                    LocalHttpTurboSolver(
                        id = it.id,
                        localTurboSolverApi = localTurboSolverApi
                    )
                }
    }
}
