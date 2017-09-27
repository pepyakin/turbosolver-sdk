package me.pepyakin.turbosolver

import android.support.v7.app.AppCompatActivity
import android.os.Bundle
import io.reactivex.disposables.Disposable

class MainActivity : AppCompatActivity() {

    private var subscription: Disposable? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val factory = LocalHttpTurboSolverFactory.create(8000)
        val solver = factory.create("""___|2__|_63
3__|__5|4_1
__1|__3|98_
___|___|_9_
___|538|___
_3_|___|___
_26|3__|5__
5_3|7__|__8
47_|__1|___""")
        subscription = solver.flatMap { it.solve() }
                .subscribe { solution, error ->
                    if (solution != null) {
                        println(solution)
                    } else {
                        println(error)
                    }
                }
    }

    override fun onDestroy() {
        super.onDestroy()
        subscription?.dispose()
    }
}
