import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.android.kotlin.multiplatform.library)
    alias(libs.plugins.vanniktech.mavenPublish)
    id("signing")
}

group = "org.siamdev"
version = "1.0.0"

kotlin {
    compilerOptions {
        freeCompilerArgs.add("-Xexpect-actual-classes")
    }

    jvm {
        compilerOptions {
            jvmTarget = JvmTarget.JVM_11
        }
    }
    androidLibrary {
        namespace = "org.siamdev.klua"
        compileSdk = libs.versions.android.compileSdk.get().toInt()
        minSdk = libs.versions.android.minSdk.get().toInt()

        withJava() // enable java compilation support
        withHostTestBuilder {}.configure {}
        withDeviceTestBuilder {
            sourceSetTreeName = "test"
        }

        compilerOptions {
            jvmTarget = JvmTarget.JVM_11
        }
    }
    
    listOf(
        iosX64(),
        iosArm64(),
        iosSimulatorArm64(),
    ).forEach { target ->
        val platform = when (target.targetName) {
            "iosArm64" -> "ios-arm64"
            "iosSimulatorArm64" -> "ios-simulator-arm64"
            "iosX64" -> "ios-simulator-x64"
            else -> error("Unsupported iOS target: ${target.targetName}")
        }
        target.compilations["main"].cinterops.create("klua") {
            defFile(project.file("src/nativeInterop/cinterop/luna.def"))
            includeDirs(
                project.file("src/nativeInterop/cinterop/headers/luna"),
                project.file("src/lib/$platform"),
            )
        }
    }


    sourceSets {
        commonMain.dependencies {
            implementation(libs.okio)
            implementation(libs.atomicfu)
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.kotlinx.datetime)
        }

        jvmMain.dependencies {
            implementation(libs.jna)
        }

        androidMain.dependencies {
            implementation("${libs.jna.get()}@aar")
        }

        commonTest.dependencies {
            implementation(libs.kotlin.test)
        }

        val androidHostTest by getting {
            dependencies {
                implementation(libs.jna)
            }
        }
    }
}

tasks.withType<Test>().configureEach {
    if (name == "testAndroidHostTest") {
        val libPath = projectDir.resolve("src/jvmMain/resources/linux-x86-64/libluna.so").absolutePath
        systemProperty("uniffi.component.luna.libraryOverride", libPath)
    }
}

mavenPublishing {
    publishToMavenCentral()

    //signAllPublications()

    coordinates(group.toString(), "lau-kmp", version.toString())

    pom {
        name = "My library"
        description = "A library."
        inceptionYear = "2024"
        url = "https://github.com/kotlin/multiplatform-library-template/"
        licenses {
            license {
                name = "XXX"
                url = "YYY"
                distribution = "ZZZ"
            }
        }
        developers {
            developer {
                id = "XXX"
                name = "YYY"
                url = "ZZZ"
            }
        }
        scm {
            url = "XXX"
            connection = "YYY"
            developerConnection = "ZZZ"
        }
    }
}

signing {
    isRequired = false
}