package me.pepyakin.turbosolver

import io.reactivex.Single

interface TurboSolver {
    fun solve(): Single<String>
    fun destroy(): Single<Unit>
}

interface TurboSolverFactory {
    fun create(grid: String): Single<TurboSolver>
}
