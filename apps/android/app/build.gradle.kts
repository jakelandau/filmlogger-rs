import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "ca.jakelandau.filmlogger"
    compileSdk = 37

    defaultConfig {
        applicationId = "ca.jakelandau.filmlogger"
        minSdk = 26
        targetSdk = 37
        versionCode = providers.gradleProperty("appVersionCode").orElse("1").get().toInt()
        versionName = providers.gradleProperty("appVersionName").orElse("0.1.0").get()
        ndk {
            // The Rust cdylib is arm64-only; keep the JNA AAR's other ABIs
            // (and their `libjnidispatch.so`) out of the APK.
            abiFilters += "arm64-v8a"
        }
    }

    val signingProperties = Properties().apply {
        val path = System.getenv("ANDROID_SIGNING_PROPERTIES")
            ?: "${System.getProperty("user.home")}/.filmlogger/release-signing.properties"
        val file = file(path)
        if (file.exists()) {
            file.inputStream().use { load(it) }
        }
    }
    val releaseStoreFile = System.getenv("ANDROID_KEYSTORE_PATH")
        ?: signingProperties.getProperty("storeFile")
    val releaseStorePassword = System.getenv("ANDROID_KEYSTORE_PASSWORD")
        ?: signingProperties.getProperty("storePassword")
    val releaseKeyAlias = System.getenv("ANDROID_KEY_ALIAS")
        ?: signingProperties.getProperty("keyAlias")
    val releaseKeyPassword = System.getenv("ANDROID_KEY_PASSWORD")
        ?: signingProperties.getProperty("keyPassword")
    val hasReleaseSigning = listOf(
        releaseStoreFile,
        releaseStorePassword,
        releaseKeyAlias,
        releaseKeyPassword,
    ).all { !it.isNullOrBlank() }

    signingConfigs {
        if (hasReleaseSigning) {
            create("release") {
                storeFile = file(releaseStoreFile!!)
                storePassword = releaseStorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }

    buildTypes {
        release {
            if (hasReleaseSigning) {
                signingConfig = signingConfigs.getByName("release")
            }
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        compose = true
    }

    lint {
        // `MainActivity` extends `androidx.activity.ComponentActivity`, whose
        // superclass chain is Kotlin. AGP 8.10.1's `Instantiatable` hierarchy
        // check fails to resolve that chain and reports a false positive; the
        // activity is public, has an empty constructor, and ultimately extends
        // `android.app.Activity`, so it is instantiable.
        disable.add("Instantiatable")
    }

    // The arm64 cdylib is copied here by `copyNativeLib` (see below).
    sourceSets["main"].jniLibs.srcDir(file("build/generated/jniLibs"))
}

kotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }
}

dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.12.01"))
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-core")
    implementation("io.coil-kt:coil-compose:2.7.0")
    implementation("io.coil-kt:coil-svg:2.7.0")
    // Uniffi's Kotlin bindings use JNA for the FFI layer. The AAR (not the
    // jar) packages `libjnidispatch.so` into the APK so JNA's
    // `System.loadLibrary("jnidispatch")` can find it on Android.
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    debugImplementation("androidx.compose.ui:ui-tooling")
    testImplementation("junit:junit:4.13.2")
}

// =====================================================================
// Rust / uniffi wiring
//
// Two independent cargo builds share the workspace `target/` dir:
//   1. A debug HOST cdylib, which uniffi-bindgen loads to introspect the
//      interface and emit the Kotlin bindings (must match the host arch).
//   2. A release arm64 cdylib, which is the library actually shipped to the
//      Android app.
// =====================================================================
val androidDir: File = rootDir
val rustDir: File = androidDir.resolve("rust")
val workspaceTarget: File = androidDir.parentFile.parentFile.resolve("target")
val hostCdylib: File = workspaceTarget.resolve(
    if (System.getProperty("os.name").startsWith("Windows")) {
        "debug/filmlogger_rs_ffi.dll"
    } else {
        "debug/libfilmlogger_rs_ffi.so"
    },
)
val armCdylib: File = workspaceTarget.resolve("aarch64-linux-android/release/libfilmlogger_rs_ffi.so")
val generatedKotlinDir: File = file("src/main/kotlin")
val generatedJniLibs: File = file("build/generated/jniLibs")
val cargoBin: String = providers
    .gradleProperty("cargoBin")
    .orElse(providers.environmentVariable("CARGO"))
    .orElse("cargo")
    .get()
val ndkRoot: String = providers
    .gradleProperty("ndkRoot")
    .orElse(providers.environmentVariable("ANDROID_NDK_HOME"))
    .orElse(providers.environmentVariable("ANDROID_NDK_LATEST_HOME"))
    .orElse("C:/Users/jakel/AppData/Local/Android/Sdk/ndk/30.0.15729638")
    .get()
// The TMDB Read Access Token, baked into the Rust cdylib at compile time
// (see `crates/filmlogger-rs/build.rs`). Gradle daemons hide shell
// environment from `Exec` tasks, so the key passes as a Gradle property:
// `-PtmdbApiKey=...`. Absent, the builds produce a keyless cdylib and the
// app fails with `NoCompiledApiKey` at `Logger.open`.
val tmdbApiKey: String = providers
    .gradleProperty("tmdbApiKey")
    .orElse("")
    .get()

// cargo-ndk is resolved as a cargo subcommand on PATH, so the cargo bin dir
// (the parent of `cargo`) is prepended to the Exec tasks' PATH.
val cargoBinDir: File? = File(cargoBin).parentFile
val pathWithCargo: String = listOfNotNull(
    System.getenv("PATH"),
    cargoBinDir?.takeIf { it.exists() }?.absolutePath,
).joinToString(File.pathSeparator)

val buildHostCdylib = tasks.register<Exec>("buildHostCdylib") {
    workingDir = rustDir
    environment("PATH", pathWithCargo)
    // Only set when non-empty: an empty value would bake an empty key in
    // (`option_env!` distinguishes absent from empty).
    if (tmdbApiKey.isNotEmpty()) {
        environment("TMDB_API_KEY", tmdbApiKey)
    }
    commandLine(cargoBin, "build")
}

val generateKotlin = tasks.register<Exec>("generateKotlin") {
    workingDir = rustDir
    commandLine(
        cargoBin, "run", "--example", "bindgen",
        "--", "generate", "-l", "kotlin",
        "--crate", "filmlogger_rs_ffi",
        "--out-dir", generatedKotlinDir.absolutePath,
        hostCdylib.absolutePath,
    )
    dependsOn(buildHostCdylib)
}

val buildArmCdylib = tasks.register<Exec>("buildArmCdylib") {
    workingDir = rustDir
    environment("NDK_HOME", ndkRoot)
    environment("PATH", pathWithCargo)
    if (tmdbApiKey.isNotEmpty()) {
        environment("TMDB_API_KEY", tmdbApiKey)
    }
    commandLine(
        cargoBin, "ndk",
        "-t", "aarch64-linux-android",
        "--platform", "26",
        "build", "--release",
    )
}

val copyNativeLib = tasks.register<Copy>("copyNativeLib") {
    from(armCdylib)
    into(generatedJniLibs.resolve("arm64-v8a"))
    dependsOn(buildArmCdylib)
}

// Generate the Kotlin bindings before any Kotlin is compiled; copy the native
// library in before the variant is packaged.
tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    dependsOn(generateKotlin)
}
tasks.named("preBuild") {
    dependsOn(copyNativeLib)
}
