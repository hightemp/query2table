import java.security.MessageDigest
import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

// Release signing key, written by `make android-keystore` or by CI from repository secrets.
// Without it, release builds are signed with the debug key so they still install for testing.
val keystoreProperties = Properties().apply {
    val propFile = rootProject.file("keystore.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

android {
    compileSdk = 36
    namespace = "com.hightemp.query2table"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "com.hightemp.query2table"
        minSdk = 24
        targetSdk = 36
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", "1").toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", "1.0")
    }
    signingConfigs {
        create("release") {
            if (!keystoreProperties.isEmpty) {
                storeFile = file(keystoreProperties.getProperty("storeFile"))
                storePassword = keystoreProperties.getProperty("storePassword")
                keyAlias = keystoreProperties.getProperty("keyAlias")
                keyPassword = keystoreProperties.getProperty("keyPassword")
            }
        }
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            // Local model servers (Ollama, llama.cpp) are often plain http on the local network.
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            signingConfig = signingConfigs.getByName(if (keystoreProperties.isEmpty) "debug" else "release")
            isMinifyEnabled = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
    packaging {
        jniLibs {
            // The bundled tools are run as programs, so they must be extracted to disk.
            useLegacyPackaging = true
        }
    }
    sourceSets["main"].jniLibs.srcDir("build/bundled-tools")
}

rust {
    rootDirRel = "../../../"
}

// Videos in research need yt-dlp, which needs Python and a JavaScript engine. youtubedl-android
// builds them for Android as executables packed like native libraries, the only files apps may
// run. Only these executables are taken, never its (GPL) Kotlin code: they run as separate
// processes, see src-tauri/src/components/bundled.rs. Same release as Cliprigo.
val bundledTools: Configuration by configurations.creating { isTransitive = false }
val bundledToolsSha256 = mapOf(
    "library-0.18.1.aar" to "579b5fb480892b1abc2b218c2089699d52759cc8d7ba256bf876453f0365faef",
)
// The architectures release builds are made for (no 32-bit x86).
val bundledAbis = listOf("arm64-v8a", "armeabi-v7a", "x86_64")

val unpackBundledTools by tasks.registering(Sync::class) {
    val aars = bundledTools.incoming.files
    val expected = bundledToolsSha256
    doFirst {
        aars.forEach { aar ->
            val actual = MessageDigest.getInstance("SHA-256").digest(aar.readBytes())
                .joinToString("") { "%02x".format(it) }
            check(expected[aar.name] == actual) { "${aar.name}: unexpected SHA-256 $actual" }
        }
    }
    from(aars.elements.map { files -> files.map { zipTree(it) } }) {
        include(bundledAbis.map { "jni/$it/**" })
        eachFile { path = path.removePrefix("jni/") }
        includeEmptyDirs = false
    }
    into(layout.buildDirectory.dir("bundled-tools"))
}
tasks.named("preBuild") { dependsOn(unpackBundledTools) }

dependencies {
    bundledTools("io.github.junkfood02.youtubedl-android:library:0.18.1@aar")
    implementation("androidx.webkit:webkit:1.14.0")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.activity:activity-ktx:1.10.1")
    implementation("com.google.android.material:material:1.12.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")