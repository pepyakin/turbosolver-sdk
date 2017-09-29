package me.pepyakin.turbosolver

import android.app.Application
import dalvik.system.DexClassLoader
import dalvik.system.DexFile

class TurboSolverApp: Application() {

    override fun onCreate() {
        super.onCreate()
        System.setProperty(
                "dexmaker.dexcache",
                getCacheDir().getPath())
    }
}
