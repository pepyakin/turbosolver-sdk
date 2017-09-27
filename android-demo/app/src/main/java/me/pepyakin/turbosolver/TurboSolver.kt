package me.pepyakin.turbosolver

import io.reactivex.Completable
import io.reactivex.Single

interface TurboSolver {
    fun solve(): Single<String>
    fun destroy(): Completable
}

interface TurboSolverFactory {
    fun create(grid: String): Single<TurboSolver>
}
