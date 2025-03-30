use askama::Template;
use regex::Regex;
use std::io::{Cursor, Write};
use time::OffsetDateTime;
use zip::write::{FileOptions, SimpleFileOptions};
use zip::{DateTime, ZipArchive, ZipWriter};

static GITIGNORE: &str = include_str!("resources/assets/zip/gitignore");

static MAVEN_WRAPPER_PROPERTIES: &str =
    include_str!("resources/assets/zip/maven-wrapper.properties");
static MVNW: &[u8] = include_bytes!("resources/assets/zip/mvnw");
static MVNW_CMD: &[u8] = include_bytes!("resources/assets/zip/mvnw.cmd");

static GRADLE_WRAPPER_JAR: &[u8] = include_bytes!("resources/assets/zip/gradle-wrapper.jar");
static GRADLE_WRAPPER_PROPERTIES: &str =
    include_str!("resources/assets/zip/gradle-wrapper.properties");
static GRADLEW: &[u8] = include_bytes!("resources/assets/zip/gradlew");
static GRADLEW_BAT: &[u8] = include_bytes!("resources/assets/zip/gradlew.bat");

pub struct ProjectFile {
    project_type: String,
    os_version: String,
    group: String,
    artifact: String,
    description: String,
    package_name: String,
}

impl ProjectFile {
    pub fn new(
        project_type: String,
        os_version: String,
        group: String,
        artifact: String,
        description: String,
        package_name: String,
    ) -> Self {
        let multiple_dots = Regex::new("\\.+").unwrap();
        let unwanted_dots = Regex::new("^\\.*|\\.*$").unwrap();
        let unwanted_in_package = Regex::new("[^a-z0-9.]").unwrap();
        let unwanted = Regex::new("[^a-z0-9.\\-]").unwrap();

        let group = multiple_dots
            .replace_all(group.to_lowercase().as_str(), ".")
            .to_string();
        let group = unwanted_dots.replace_all(group.as_str(), "").to_string();
        let group = unwanted.replace_all(group.as_str(), "").to_string();

        let artifact = multiple_dots
            .replace_all(artifact.to_lowercase().as_str(), ".")
            .to_string();
        let artifact = unwanted_dots.replace_all(artifact.as_str(), "").to_string();
        let artifact = unwanted.replace_all(artifact.as_str(), "").to_string();

        let package_name = multiple_dots
            .replace_all(package_name.to_lowercase().as_str(), ".")
            .to_string();
        let package_name = unwanted_dots
            .replace_all(package_name.as_str(), "")
            .to_string();
        let package_name = unwanted_in_package
            .replace_all(package_name.as_str(), "")
            .to_string();

        Self {
            project_type,
            os_version,
            group,
            artifact,
            description,
            package_name,
        }
    }

    pub fn to_zip_archive(&self) -> Result<ZipArchive<Cursor<Vec<u8>>>, ()> {
        fn recursive_create_dir(
            path: &str,
            zip: &mut ZipWriter<&mut Cursor<Vec<u8>>>,
            options: FileOptions<()>,
        ) {
            let segments = path.split('/').collect::<Vec<_>>();

            if segments.is_empty() {
                return;
            }

            (1..=segments.len()).for_each(|i| {
                let path = segments
                    .iter()
                    .take(i)
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join("/");
                println!("{}", path);
                let _ = zip.add_directory(path, options);
            });
        }

        let mut archive = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(&mut archive);
        let now = OffsetDateTime::now_utc();
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::DEFLATE)
            .last_modified_time(DateTime::try_from(now).unwrap());

        if self.project_type == "gradle" {
            // Gradle
            zip.start_file("build.gradle", options).map_err(|_| ())?;
            zip.write(
                BuildGradleTemplate {
                    group: self.group.to_string(),
                    os_version: self.os_version.to_string(),
                    description: self.description.to_string(),
                }
                .to_string()
                .as_bytes(),
            )
            .map_err(|_| ())?;

            zip.start_file("settings.gradle", options).map_err(|_| ())?;
            zip.write(format!("rootProject.name = '{}'", self.artifact).as_bytes())
                .map_err(|_| ())
                .map_err(|_| ())?;

            recursive_create_dir("gradle/wrapper", &mut zip, options);
            zip.start_file("gradle/wrapper/gradle-wrapper.jar", options)
                .map_err(|_| ())?;
            zip.write(GRADLE_WRAPPER_JAR)
                .map_err(|_| ())
                .map_err(|_| ())?;

            zip.start_file("gradle/wrapper/gradle-wrapper.properties", options)
                .map_err(|_| ())?;
            zip.write(GRADLE_WRAPPER_PROPERTIES.as_bytes())
                .map_err(|_| ())
                .map_err(|_| ())?;

            zip.start_file("gradlew", options.unix_permissions(0o755))
                .map_err(|_| ())?;
            zip.write(GRADLEW).map_err(|_| ()).map_err(|_| ())?;

            zip.start_file("gradlew.bat", options).map_err(|_| ())?;
            zip.write(GRADLEW_BAT).map_err(|_| ()).map_err(|_| ())?;
        } else {
            // Maven
            zip.start_file("pom.xml", options).map_err(|_| ())?;
            zip.write(
                PomXmlTemplate {
                    group: self.group.to_string(),
                    artifact: self.artifact.to_string(),
                    os_version: self.os_version.to_string(),
                    description: self.description.to_string(),
                }
                .to_string()
                .as_bytes(),
            )
            .map_err(|_| ())?;

            recursive_create_dir(".mvn/wrapper", &mut zip, options);
            zip.start_file(".mvn/wrapper/maven-wrapper.properties", options)
                .map_err(|_| ())?;
            zip.write(MAVEN_WRAPPER_PROPERTIES.as_bytes())
                .map_err(|_| ())
                .map_err(|_| ())?;

            zip.start_file("mvnw", options.unix_permissions(0o755))
                .map_err(|_| ())?;
            zip.write(MVNW).map_err(|_| ()).map_err(|_| ())?;

            zip.start_file("mvnw.cmd", options).map_err(|_| ())?;
            zip.write(MVNW_CMD).map_err(|_| ()).map_err(|_| ())?;
        }

        zip.start_file("HELP.md", options).map_err(|_| ())?;
        zip.write(
            HelpMdTemplate {
                package_name: self.package_name.to_string(),
            }
            .to_string()
            .as_bytes(),
        )
        .map_err(|_| ())?;

        zip.start_file(".gitignore", options).map_err(|_| ())?;
        zip.write(GITIGNORE.as_bytes())
            .map_err(|_| ())
            .map_err(|_| ())?;

        recursive_create_dir("libs", &mut zip, options);
        zip.start_file("libs/.gitkeep", options).map_err(|_| ())?;
        zip.write("Place onkostar-api.jar here".to_string().as_bytes())
            .map_err(|_| ())?;

        recursive_create_dir("src/main/resources", &mut zip, options);
        zip.start_file("src/main/resources/onkostar-config.properties", options)
            .map_err(|_| ())?;
        zip.write(format!("onkostar-api={}", self.os_version).as_bytes())
            .map_err(|_| ())?;

        recursive_create_dir(
            "src/main/resources/de/itc/onkostar/library",
            &mut zip,
            options,
        );
        zip.start_file(
            "src/main/resources/de/itc/onkostar/library/moduleContext.xml",
            options,
        )
        .map_err(|_| ())?;
        zip.write(
            ModuleContextXmlTemplate {
                package_name: self.package_name.to_string(),
            }
            .to_string()
            .as_bytes(),
        )
        .map_err(|_| ())?;

        recursive_create_dir(
            &format!("src/main/java/{}", self.package_name.replace(".", "/")),
            &mut zip,
            options,
        );
        zip.start_file(
            format!(
                "src/main/java/{}/ExampleAnalyzer.java",
                self.package_name.replace(".", "/")
            )
            .as_str(),
            options,
        )
        .map_err(|_| ())?;
        zip.write(
            ExampleAnalyzerJavaTemplate {
                package_name: self.package_name.to_string(),
            }
            .to_string()
            .as_bytes(),
        )
        .map_err(|_| ())?;

        recursive_create_dir(
            &format!("src/test/java/{}", self.package_name.replace(".", "/")),
            &mut zip,
            options,
        );
        zip.start_file(
            format!(
                "src/test/java/{}/ExampleAnalyzerTest.java",
                self.package_name.replace(".", "/")
            )
            .as_str(),
            options,
        )
        .map_err(|_| ())?;
        zip.write(
            ExampleAnalyzerTestJavaTemplate {
                package_name: self.package_name.to_string(),
            }
            .to_string()
            .as_bytes(),
        )
        .map_err(|_| ())?;

        zip.finish().map_err(|_| ())?;

        archive.flush().unwrap();
        let zip_archive = ZipArchive::new(archive).unwrap();
        Ok(zip_archive)
    }
}

#[derive(Template)]
#[template(path = "zip/pom.xml")]
struct PomXmlTemplate {
    group: String,
    artifact: String,
    os_version: String,
    description: String,
}

#[derive(Template)]
#[template(path = "zip/build.gradle.txt")]
struct BuildGradleTemplate {
    group: String,
    os_version: String,
    description: String,
}

#[derive(Template)]
#[template(path = "zip/moduleContext.xml")]
struct ModuleContextXmlTemplate {
    package_name: String,
}

#[derive(Template)]
#[template(path = "zip/ExampleAnalyzer.txt")]
struct ExampleAnalyzerJavaTemplate {
    package_name: String,
}

#[derive(Template)]
#[template(path = "zip/ExampleAnalyzerTest.txt")]
struct ExampleAnalyzerTestJavaTemplate {
    package_name: String,
}

#[derive(Template)]
#[template(path = "zip/HELP.md")]
struct HelpMdTemplate {
    package_name: String,
}
