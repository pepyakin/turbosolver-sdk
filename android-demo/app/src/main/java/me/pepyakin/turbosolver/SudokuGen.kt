package me.pepyakin.turbosolver

import io.reactivex.Single
import org.mariuszgromada.math.janetsudoku.SudokuGenerator

fun generateSudoku(): Single<String> =
        Single.fromCallable {
            val generator = SudokuGenerator(
                    SudokuGenerator.PARAM_GEN_RND_BOARD,
                    SudokuGenerator.PARAM_DO_NOT_SOLVE
            )
            val board = generator.generate()
            board
                    .fold(StringBuilder()) { acc, row ->
                        row.foldIndexed(acc) { index, acc, digit ->
                            if (index != 0 && index % 3 == 0) {
                                acc.append("|")
                            }
                            when (digit) {
                                0 -> acc.append('_')
                                else -> acc.append(digit)
                            }
                        }
                        acc.append("\n")
                    }
                    .toString()
        }
