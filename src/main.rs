extern crate core;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::{Context, Result};
use xmltree::Element;
use zip::read::ZipFile;
use zip::ZipArchive;

// /Users/wanglunhui/Downloads/jedis-4.3.2.jar
// /Users/wanglunhui/Server/Maven/repository/com/gdnm/gdnm-common/2.0.5/gdnm-common-2.0.5.pom
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // 第 0 个为程序名
    //let program_name = &args[0];

    let input_files = args.iter().skip(1).collect::<Vec<_>>(); // 第 0 个为程序名，需要跳过

    for input_file in input_files {
        let path = Path::new(input_file);
        let file = File::open(input_file).with_context(|| format!("读取文件失败: {}", input_file))?;

        let full_path = path.to_str().expect("无法转为可读路径");
        let extension = path.extension().unwrap();

        match extension.to_str() {
            None => {}
            Some(extension) => {
                if extension.eq("jar"){
                    let mut archive = ZipArchive::new(file)?;

                    for i in 0..archive.len() {
                        let mut jar_file = archive.by_index(i)?;
                        let jar_path = jar_file.name();
                        if jar_path.starts_with("META-INF/maven/") && jar_path.ends_with("pom.properties"){
                            let (group_id, artifact_id, version) = parse_jar_pom_properties(&mut jar_file);
                            println!("mvn install:install-file -Dfile={} -DgroupId={} -DartifactId={} -Dversion={} -Dpackaging=jar", full_path, group_id, artifact_id, version);
                        }
                    }
                }else if extension.eq("pom"){
                    let (group_id, artifact_id, version) = parse_pom(&file);
                    println!("mvn install:install-file -Dfile={} -DgroupId={} -DartifactId={} -Dversion={} -Dpackaging=pom", full_path, group_id, artifact_id, version);
                }else{
                    panic!("不支持该类型: {}", extension);
                }
            }
        }
    }

    Ok(())
}

pub fn parse_jar_pom_properties(jar_file: &mut ZipFile) -> (String,String,String) {
    let mut group_id:Option<String> = None;
    let mut artifact_id:Option<String> = None;
    let mut version:Option<String> = None;

    let reader = BufReader::new(jar_file);
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with('#') || line.starts_with('!') || line.trim().is_empty() { // 忽略注释行和空行
            continue;
        }
        let parts: Vec<_> = line.splitn(2, '=').collect();
        if parts.len() != 2 { // 忽略无效行
            continue;
        }
        let key = parts[0].trim();
        let value = parts[1].trim();

        if key.eq("groupId") {
            group_id = Some(value.into());
        }else if key.eq("artifactId") {
            artifact_id = Some(value.into());
        }else if key.eq("version"){
            version = Some(value.into());
        }
    }

    (group_id.unwrap_or("".into()), artifact_id.unwrap_or("".into()), version.unwrap_or("".into()))
}

pub fn parse_pom(jar_file: &File) -> (String,String,String) {
    let project = Element::parse(jar_file).unwrap();
    let group_id_node = project.get_child("groupId");
    let artifact_id_node = project.get_child("artifactId");
    let version_node = project.get_child("version");

    let mut group_id:Option<String> = None;
    let mut artifact_id:Option<String> = None;
    let mut version:Option<String> = None;

    if let Some(v) = group_id_node{
        let name = &v.name;
        group_id = Some(String::from(name));
    }
    if let Some(v) = artifact_id_node{
        let name = &v.name;
        artifact_id = Some(String::from(name));
    }
    if let Some(v) = version_node{
        let name = &v.name;
        version = Some(String::from(name));
    }

    (group_id.unwrap_or("".into()), artifact_id.unwrap_or("".into()), version.unwrap_or("".into()))
}
