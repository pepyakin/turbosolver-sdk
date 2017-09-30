package me.pepyakin.turbosolver

import android.annotation.SuppressLint
import android.support.v7.app.AppCompatActivity
import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import io.reactivex.Single
import io.reactivex.android.schedulers.AndroidSchedulers
import io.reactivex.disposables.Disposable
import io.reactivex.schedulers.Schedulers


class MainActivity : AppCompatActivity() {

    private var subscription: Disposable? = null

    private val generateBtn by lazy {
        findViewById(R.id.main_generate_btn) as Button
    }

    private val puzzleTextView by lazy {
        findViewById(R.id.main_puzzle_textview) as TextView
    }

    private val solutionTextView by lazy {
        findViewById(R.id.main_solution_textview) as TextView
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        generateBtn.setOnClickListener {
            generateRandomSudoku()
        }
    }

    @SuppressLint("SetTextI18n")
    private fun generateRandomSudoku() {
        subscription?.dispose()

        subscription = generateSudoku()
                .subscribeOn(Schedulers.computation())
                .observeOn(AndroidSchedulers.mainThread())
                .doOnSubscribe {
                    puzzleTextView.text = "Generating..."
                    solutionTextView.text = "Waiting for generated sudoku..."
                }
                .doOnSuccess { sudoku ->
                    puzzleTextView.text = sudoku
                    solutionTextView.text = "Solving..."
                }
                .observeOn(Schedulers.computation())
                .flatMap { sudoku ->
                    solveSudoku(sudoku)
                }
                .observeOn(AndroidSchedulers.mainThread())
                .subscribe { solution, error ->
                    solutionTextView.text = if (solution != null) {
                        "Solution:\n" + solution
                    } else {
                        // Blow up the app!
                        kotlin.error(error)
                    }
                }
    }

    private fun solveSudoku(grid: String): Single<String> {
        // val factory = CapnpTurboSolverFactory.create()
        // val factory = LocalHttpTurboSolverFactory.create()
        val factory = JnrTurboSolverFactory.create(this)

        val solverFuture = factory.create(grid)
        return solverFuture
                .flatMap { solver ->
                    val futureSolution = solver.solve()

                    futureSolution.flatMap { solution ->
                        solver.destroy()
                                .toSingle { Unit }
                                .map { solution }
                    }
                }
    }

    override fun onDestroy() {
        super.onDestroy()
        subscription?.dispose()
    }
}
