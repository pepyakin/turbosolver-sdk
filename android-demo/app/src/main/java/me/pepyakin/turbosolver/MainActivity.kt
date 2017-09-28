package me.pepyakin.turbosolver

import android.support.v7.app.AppCompatActivity
import android.os.Bundle
import android.widget.TextView
import io.reactivex.Single
import io.reactivex.SingleSource
import io.reactivex.android.schedulers.AndroidSchedulers
import io.reactivex.disposables.Disposable
import me.pepyakin.turbosolver.capnp.CapnpTurboSolverFactory

private val sudokuGrid = """___|2__|_63
3__|__5|4_1
__1|__3|98_
___|___|_9_
___|538|___
_3_|___|___
_26|3__|5__
5_3|7__|__8
47_|__1|___"""

class MainActivity : AppCompatActivity() {

    private var subscription: Disposable? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val textView = findViewById(R.id.main_text) as TextView

        val factory = CapnpTurboSolverFactory.create()
        val solverFuture = factory.create(sudokuGrid)
        subscription = solverFuture
                .flatMap { solver ->
                    val futureSolution = solver.solve()

                    futureSolution.flatMap { solution ->
                        solver.destroy()
                                .toSingle { Unit }
                                .map { solution }
                    }
                }
                .observeOn(AndroidSchedulers.mainThread())
                .subscribe { solution, error ->
                    textView.text = if (solution != null) {
                        "Solution:\n" + solution
                    } else {
                        "Error:\n" + error
                    }
                }
    }

    override fun onDestroy() {
        super.onDestroy()
        subscription?.dispose()
    }
}
