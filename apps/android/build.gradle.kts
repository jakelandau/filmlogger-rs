plugins {
    id("com.android.application") version "9.5.1" apply false
    // Built-in Kotlin (AGP 9) compiles Kotlin; the Compose compiler still needs
    // its own plugin, at AGP's bundled Kotlin Gradle plugin version.
    id("org.jetbrains.kotlin.plugin.compose") version "2.2.10" apply false
}
