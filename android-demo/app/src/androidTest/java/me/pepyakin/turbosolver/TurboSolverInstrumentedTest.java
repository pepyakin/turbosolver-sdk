package me.pepyakin.turbosolver;

import android.content.Context;
import android.support.test.InstrumentationRegistry;
import android.support.test.runner.AndroidJUnit4;

import org.junit.Test;
import org.junit.runner.RunWith;

import io.reactivex.Single;
import me.pepyakin.turbosolver.capnp.CapnpTurboSolverFactory;

import static org.junit.Assert.*;

/**
 * Instrumentation test, which will execute on an Android device.
 *
 * @see <a href="http://d.android.com/tools/testing">Testing documentation</a>
 */
@RunWith(AndroidJUnit4.class)
public class TurboSolverInstrumentedTest {
    @Test
    public void testHttpTurboSolver() throws Exception {
        CapnpTurboSolverFactory factory = CapnpTurboSolverFactory.create();
        for (int i = 0; i < 100; i++) {
            testSolver(factory);
        }
    }

    @Test
    public void testCapnpTurboSolver() throws Exception {
        LocalHttpTurboSolverFactory factory = LocalHttpTurboSolverFactory.create(8000);
        for (int i = 0; i < 100; i++) {
            testSolver(factory);
        }
    }

    private void testSolver(TurboSolverFactory factory) {
        String sudoku = SudokuGenKt.generateSudoku().blockingGet();
        TurboSolver turboSolver = factory.create(sudoku).blockingGet();
        String solution = turboSolver.solve().blockingGet();
        turboSolver.destroy().blockingAwait();
    }
}
