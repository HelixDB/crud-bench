#![cfg(feature = "helixdb")]

use crate::database::Database;
use crate::docker::DockerParams;
use crate::engine::{BenchmarkClient, BenchmarkEngine};
use crate::valueprovider::Columns;
use crate::{Benchmark, KeyType, Projection, Scan};
use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::io::{Read, Write};
use std::net::TcpStream;

pub const DEFAULT: &str = "http://localhost:6969";

pub(crate) const fn docker(options: &Benchmark) -> DockerParams {
    match options.database {
        Database::Helixdb => DockerParams {
            image: "helixdb/helixdb:latest",
            pre_args: "-p 6969:6969",
            post_args: "",
        },
        _ => unreachable!(),
    }
}

pub(crate) struct HelixDBClientProvider {
    endpoint: String,
    project_path: PathBuf,
}

impl BenchmarkEngine<HelixDBClient> for HelixDBClientProvider {
    /// Initiates a new datastore benchmarking engine
    async fn setup(_: KeyType, _columns: Columns, options: &Benchmark) -> Result<Self> {
        // Get the custom endpoint if specified
        let endpoint = options.endpoint.as_deref().unwrap_or(DEFAULT).to_string();
        
        // Create a temporary directory for the HelixDB project
        let project_path = PathBuf::from("~/crud-bench/helixdb-queries");
        
        // Create the queries.hx file with the necessary CRUD operations
        
        // // Deploy the queries locally
        // Command::new("helix")
        //     .arg("deploy")
        //     .arg("--local")
        //     .current_dir(&project_path)
        //     .output()?;
        
        Ok(Self {
            endpoint,
            project_path,
        })
    }
    
    /// Creates a new client for this benchmarking engine
    async fn create_client(&self) -> Result<HelixDBClient> {
        Ok(HelixDBClient::new(self.endpoint.clone()))
    }
}

pub(crate) struct HelixDBClient {
    endpoint: String,
}

impl HelixDBClient {
    fn new(endpoint: String) -> Self {
        Self {
            endpoint,
        }
    }

    fn extract_host_and_port(endpoint: &str) -> (String, u16) {
        let url = endpoint.replace("http://", "");
        let parts: Vec<&str> = url.split(':').collect();
        let host = parts[0].to_string();
        let port = parts.get(1).and_then(|p| p.parse::<u16>().ok()).unwrap_or(80);
        (host, port)
    }

    fn make_request(&self, path: &str, method: &str, body: Option<&str>) -> Result<String> {
        let (host, port) = Self::extract_host_and_port(&self.endpoint);
        let mut stream = TcpStream::connect((host.clone(), port))?;
        
        let body_str = body.unwrap_or("");
        let content_length = body_str.len();
        
        let request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccept: application/json\r\n\r\n{}",
            method, path, host, content_length, body_str
        );
        
        stream.write_all(request.as_bytes())?;
        
        let mut response = String::new();
        let mut buffer = [0; 4096];
        loop {
            let bytes_read = stream.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            response.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));
        }
        
        // Extract the body from the response
        let body_start = response.find("\r\n\r\n").unwrap_or(0) + 4;
        let body = response[body_start..].to_string();
        
        Ok(body)
    }
}

impl BenchmarkClient for HelixDBClient {
    async fn startup(&self) -> Result<()> {
        // No specific startup needed for HelixDB
        Ok(())
    }

    async fn create_u32(&self, key: u32, val: Value) -> Result<()> {
        let path = "/create_record";
        let data = extract_string_field(&val)?;
        let body = serde_json::json!({
            "id": key.to_string(),
            "data": data
        }).to_string();
        
        self.make_request(path, "POST", Some(&body))?;
        Ok(())
    }

    async fn create_string(&self, key: String, val: Value) -> Result<()> {
        let path = "/create_record";
        let data = extract_string_field(&val)?;
        let body = serde_json::json!({
            "id": key,
            "data": data
        }).to_string();
        
        self.make_request(path, "POST", Some(&body))?;
        Ok(())
    }

    async fn read_u32(&self, key: u32) -> Result<()> {
        let path = "/read_record";
        let body = serde_json::json!({
            "id": key.to_string()
        }).to_string();
        
        self.make_request(path, "POST", Some(&body))?;
        Ok(())
    }

    async fn read_string(&self, key: String) -> Result<()> {
        let path = "/read_record";
        let body = serde_json::json!({
            "id": key
        }).to_string();
        
        self.make_request(path, "POST", Some(&body))?;
        Ok(())
    }

    async fn update_u32(&self, key: u32, val: Value) -> Result<()> {
        let path = "/update_record";
        let data = extract_string_field(&val)?;
        let body = serde_json::json!({
            "id": key.to_string(),
            "data": data
        }).to_string();
        
        self.make_request(path, "POST", Some(&body))?;
        Ok(())
    }

    async fn update_string(&self, key: String, val: Value) -> Result<()> {
        let path = "/update_record";
        let data = extract_string_field(&val)?;
        let body = serde_json::json!({
            "id": key,
            "data": data
        }).to_string();
        
        self.make_request(path, "POST", Some(&body))?;
        Ok(())
    }

    async fn delete_u32(&self, key: u32) -> Result<()> {
        let path = "/delete_record";
        let body = serde_json::json!({
            "id": key.to_string()
        }).to_string();
        
        self.make_request(path, "POST", Some(&body))?;
        Ok(())
    }

    async fn delete_string(&self, key: String) -> Result<()> {
        let path = "/delete_record";
        let body = serde_json::json!({
            "id": key
        }).to_string();
        
        self.make_request(path, "POST", Some(&body))?;
        Ok(())
    }

    async fn scan_u32(&self, scan: &Scan) -> Result<usize> {
        self.scan(scan).await
    }

    async fn scan_string(&self, scan: &Scan) -> Result<usize> {
        self.scan(scan).await
    }
}

impl HelixDBClient {
    async fn scan(&self, scan: &Scan) -> Result<usize> {
        // Extract parameters
        let limit = scan.limit.unwrap_or(100) as i64;
        let offset = scan.start.unwrap_or(0) as i64;
        
        // Perform the relevant projection scan type
        match scan.projection()? {
            Projection::Id | Projection::Full => {
                let path = "/scan_records";
                let body = serde_json::json!({
                    "limit": offset + limit,
                    "offset": offset
                }).to_string();
                
                let response = self.make_request(path, "POST", Some(&body))?;
                let result: Value = serde_json::from_str(&response)?;
                let count = result.as_array().map(|arr| arr.len()).unwrap_or(0);
                Ok(count)
            }
            Projection::Count => {
                let path = "/count_records";
                let response = self.make_request(path, "POST", None)?;
                let result: Value = serde_json::from_str(&response)?;
                let count = result.as_i64().unwrap_or(0) as usize;
                Ok(count)
            }
        }
    }
}

// Helper function to extract the string field from a JSON value
fn extract_string_field(val: &Value) -> Result<String> {
    if let Some(obj) = val.as_object() {
        // Find the first string field in the object
        for (_, value) in obj {
            if let Some(s) = value.as_str() {
                return Ok(s.to_string());
            }
        }
    }
    
    // If no string field is found, return the value as a string
    Ok(val.to_string())
}
