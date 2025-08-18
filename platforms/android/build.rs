use std::{env, path::PathBuf};

const JAVA_FILE_RELATIVE_PATH: &str = "java/dev/accesskit/android/Delegate.java";

fn main() {
    if env::var("CARGO_FEATURE_EMBEDDED_DEX").is_ok() {
        println!("cargo:rerun-if-changed={JAVA_FILE_RELATIVE_PATH}");

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let java_file_path =
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(JAVA_FILE_RELATIVE_PATH);
        let android_jar_path =
            android_build::android_jar(None).expect("Failed to find android.jar");

        let javac_succeeded = android_build::JavaBuild::new()
            .class_path(&android_jar_path)
            .java_source_version(8)
            .java_target_version(8)
            .deprecation(true)
            .classes_out_dir(&out_dir)
            .file(java_file_path)
            .compile()
            .expect("Failed to acquire exit status for javac invocation")
            .success();
        if !javac_succeeded {
            panic!("javac invocation failed");
        }

        let d8_jar_path = android_build::android_d8_jar(None).expect("Failed to find d8.jar");

        let dexer_succeeded = android_build::Dexer::new()
            .class_path(&android_jar_path)
            .android_d8_jar(d8_jar_path)
            .android_jar(android_jar_path)
            .out_dir(&out_dir)
            .collect_classes(out_dir)
            .unwrap()
            .run()
            .expect("Failed to acquire exit status for java d8.jar invocation")
            .success();
        if !dexer_succeeded {
            panic!("java d8.jar invocation failed");
        }
    }
}
