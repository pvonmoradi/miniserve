mod fixtures;

use fixtures::{server, Error, TestServer, DIRECTORIES, FILES};
use rstest::rstest;
use select::predicate::Attr;
use select::{document::Document, node::Node};
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::PathBuf;

#[rstest]
/// Do not show readme contents by default
fn no_readme_contents(server: TestServer) -> Result<(), Error> {
    let body = reqwest::blocking::get(server.url())?.error_for_status()?;
    let parsed = Document::from_read(body)?;
    for &file in FILES {
        assert!(parsed.find(|x: &Node| x.text() == file).next().is_some());
    }
    for &dir in DIRECTORIES {
        assert!(parsed.find(|x: &Node| x.text() == dir).next().is_some());
    }
    assert!(parsed.find(Attr("id", "readme")).next().is_none());
    assert!(parsed.find(Attr("id", "readme-filename")).next().is_none());
    assert!(parsed.find(Attr("id", "readme-contents")).next().is_none());

    Ok(())
}

#[rstest(
    readme_name,
    case("Readme.md"),
    case("readme.md"),
    case("README.md"),
    case("README.MD"),
    case("ReAdMe.Md")
)]

/// Show readme contents when told to if there is readme.md/README.md file in root
fn show_root_readme_contents(
    #[with(&["--readme"])] server: TestServer,
    readme_name: &str,
) -> Result<(), Error> {
    let readme_path = write_readme_contents(server.path().to_path_buf(), readme_name);
    let body = reqwest::blocking::get(server.url())?.error_for_status()?;
    let parsed = Document::from_read(body)?;

    for &file in FILES {
        assert!(parsed.find(|x: &Node| x.text() == file).next().is_some());
    }
    assert_readme_contents(&parsed, &readme_name);
    remove_file(readme_path).unwrap();
    Ok(())
}

#[rstest(
    readme_name,
    case("Readme.md"),
    case("readme.md"),
    case("README.md"),
    case("README.MD"),
    case("ReAdMe.Md")
)]
/// Show readme contents when told to if there is readme.md/README.md file in directories
fn show_nested_readme_contents(
    #[with(&["--readme"])] server: TestServer,
    readme_name: &str,
) -> Result<(), Error> {
    for dir in DIRECTORIES {
        let readme_path = write_readme_contents(server.path().join(dir), readme_name);
        let body = reqwest::blocking::get(server.url().join(dir)?)?.error_for_status()?;
        let parsed = Document::from_read(body)?;

        for &file in FILES {
            assert!(parsed.find(|x: &Node| x.text() == file).next().is_some());
        }
        assert_readme_contents(&parsed, &readme_name);
        remove_file(readme_path).unwrap();
    }
    Ok(())
}

fn write_readme_contents(path: PathBuf, filename: &str) -> PathBuf {
    let readme_path = path.join(filename);
    let mut readme_file = File::create(&readme_path).unwrap();
    readme_file
        .write(format!("Contents of {}", filename).to_string().as_bytes())
        .expect("Couldn't write readme");
    readme_path
}

fn assert_readme_contents(parsed_dom: &Document, filename: &str) {
    assert!(parsed_dom.find(Attr("id", "readme")).next().is_some());
    assert!(parsed_dom
        .find(Attr("id", "readme-filename"))
        .next()
        .is_some());
    assert!(
        parsed_dom
            .find(Attr("id", "readme-filename"))
            .next()
            .unwrap()
            .text()
            == filename
    );
    assert!(parsed_dom
        .find(Attr("id", "readme-contents"))
        .next()
        .is_some());
    assert!(
        parsed_dom
            .find(Attr("id", "readme-contents"))
            .next()
            .unwrap()
            .text()
            .trim()
            == format!("Contents of {}", filename)
    );
}
