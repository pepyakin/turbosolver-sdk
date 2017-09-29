package me.pepyakin.turbosolver

import android.app.Application

class TurboSolverApp: Application() {

    override fun onCreate() {
        super.onCreate()
        System.setProperty(
                "jnrffi.dexcache",
                cacheDir.path)
    }
}
