import javax.inject.Inject
import org.gradle.process.ExecOperations

plugins {
    alias(libs.plugins.android.application)
}

// Override with e.g. `-PrustAbis=arm64-v8a,x86_64`; the matching Rust
// targets must be installed.
val rustAbis = (findProperty("rustAbis") as String? ?: "arm64-v8a").split(',')

android {
    namespace = "dev.accesskit.helloworld"
    compileSdk = 37

    defaultConfig {
        applicationId = "dev.accesskit.helloworld"
        minSdk = 30
        targetSdk = 37
        versionCode = 1
        versionName = "1.0"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    packaging {
        jniLibs {
            keepDebugSymbols += "**/*.so"
        }
    }
}

abstract class CargoNdkBuild @Inject constructor(private val execOperations: ExecOperations) :
    DefaultTask() {
    @get:Input abstract val abis: ListProperty<String>
    @get:Input abstract val apiLevel: Property<Int>
    @get:Input abstract val release: Property<Boolean>
    @get:Input abstract val sdkDir: Property<String>
    @get:InputFile abstract val cargoManifest: RegularFileProperty
    @get:OutputDirectory abstract val outputDir: DirectoryProperty

    init {
        doNotTrackState("cargo tracks its own inputs")
    }

    @TaskAction
    fun build() {
        val manifest = cargoManifest.get().asFile
        val outputDir = outputDir.get().asFile
        outputDir.deleteRecursively()
        outputDir.mkdirs()
        execOperations.exec {
            workingDir = manifest.parentFile
            // So cargo-ndk can find an NDK inside the SDK.
            environment("ANDROID_HOME", sdkDir.get())
            commandLine = buildList {
                add("cargo")
                add("ndk")
                add("--platform")
                add(apiLevel.get().toString())
                add("--output-dir")
                add(outputDir.absolutePath)
                for (abi in abis.get()) {
                    add("--target")
                    add(abi)
                }
                add("build")
                add("--example")
                add("hello_world")
                if (release.get()) {
                    add("--release")
                }
            }
        }
    }
}

androidComponents {
    onVariants { variant ->
        val cargoBuild =
            tasks.register<CargoNdkBuild>(
                "cargoBuild${variant.name.replaceFirstChar { it.uppercase() }}"
            ) {
                abis = rustAbis
                apiLevel = variant.minSdk.apiLevel
                release = variant.buildType == "release"
                sdkDir = sdkComponents.sdkDirectory.map { it.asFile.absolutePath }
                cargoManifest = layout.projectDirectory.file("../../../Cargo.toml")
                outputDir = layout.buildDirectory.dir("rustJniLibs/${variant.name}")
            }
        variant.sources.jniLibs?.addGeneratedSourceDirectory(cargoBuild, CargoNdkBuild::outputDir)
    }
}
