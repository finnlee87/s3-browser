use std::path::Path;

use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};
use tokio::fs::File;

pub struct S3Operator {
    access_key: String,
    secret_key: String,
    endpoint_url: String
}

fn bytes_to_readable(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

impl S3Operator {
    pub fn new() -> Self {
        S3Operator {
            access_key: "".to_string(),
            secret_key: "".to_string(),
            endpoint_url: "".to_string()
        }
    }

    pub async fn init(&mut self, access_key: &str, secret_key: &str, endpoint_url: &str) {
        if (self.access_key != access_key) || (self.secret_key != access_key) || (self.endpoint_url != endpoint_url) {     
            self.access_key = access_key.into();
            self.secret_key = secret_key.into();
            self.endpoint_url = endpoint_url.into();
        }
    }

    pub async fn create_bucket(&self, bucket: &str, region: &str) -> Result<(), String> {
        let region: Region = Region::Custom {
            region: region.to_owned(),
            endpoint: self.endpoint_url.to_owned(),
        };
        let credentials = Credentials::new(Some(&self.access_key), Some(&self.secret_key), None, None, None).unwrap();
        let mut conf = BucketConfiguration::default();
        conf.set_location_constraint(region.clone());
        let response = Bucket::create_with_path_style(
            bucket,
            region.clone(),
            credentials.clone(),
            conf,
        ).await;
        match response {
            Ok(_) => {
                Ok(())
            }
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn delete_bucket(&self, bucket: &str) -> Result<(), String> {
        let region: Region = Region::Custom {
            region: "".to_owned(),
            endpoint: self.endpoint_url.to_owned(),
        };
        let credentials = Credentials::new(Some(&self.access_key), Some(&self.secret_key), None, None, None).unwrap();
        let bucket = Bucket::new(bucket, region.clone(), credentials.clone()).unwrap().with_path_style();
        let response = bucket.delete().await;

        match response {
            Ok(_) => {
                Ok(())
            }
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    pub async fn list_buckets(&self) -> Result<Vec<String>, String>{
        let region = Region::Custom {
            region: "us-east-1".to_owned(),
            endpoint: self.endpoint_url.to_owned(),
        };
        let credentials = Credentials::new(Some(&self.access_key), Some(&self.secret_key), None, None, None).unwrap();

        let response = Bucket::list_buckets(region, credentials).await;

        match response {
            Ok(bucket_response) => {
                let mut names = Vec::new();
                for bucket_name in bucket_response.bucket_names() {
                    names.push(bucket_name);
                }
                Ok(names)
            }   
            Err(error) => {
                println!("{}", error);
                Err(error.to_string())
            }
        }

        // match &self.s3_client {
        //     Some(client) => {
        //         let resp = client.list_buckets().send().await;
        //         match resp {
        //             Ok(output) => {
        //                 let buckets = output.buckets();
        //                 let mut names = Vec::new();
        //                 for bucket in buckets {
        //                     let bucket_name = bucket.name().unwrap().to_string();
        //                     println!("{}", bucket_name);
        //                     names.push(bucket_name);
                            
        //                 }
        //                 return Ok(names)
        //             }
        //             Err(error) => {
        //                 Err(error.to_string())
        //             }
        //         }
        //     }
        //     None => {
        //         Err(String::from("no client"))
        //     }
        // }
    }

    pub async fn list_objects(&self, bucket_name: &str) -> Result<Vec<String>, String> {

        let region = Region::Custom {
            region: "us-east-1".to_owned(),
            endpoint: self.endpoint_url.to_owned(),
        };
        let credentials = Credentials::new(Some(&self.access_key), Some(&self.secret_key), None, None, None).unwrap();
        let bucket = Bucket::new(bucket_name, region.clone(), credentials.clone()).unwrap().with_path_style();
        // let bucket = Bucket::create_with_path_style(bucket_name, region.clone(), credentials.clone(), config);

        // Async variant with `tokio` or `async-std` features
        let result = bucket.list(String::default(), None).await;

        match result {
            Ok(list_results) => {
                let mut data = Vec::new();
                for list_result in list_results {
                    for object in list_result.contents {
                        let key = object.key;
                        let last_modified = object.last_modified;
                        data.push(format!("{},{},{}", key, last_modified, bytes_to_readable(object.size)));
                    }
                }
                Ok(data)
            }   
            Err(error) => {
                println!("{}", error);
                Err(error.to_string())
            }
        }
    }

    pub async fn create_folder(&self, folder_name: &str, folder_path: &str) -> Result<(), String> {
        let region = Region::Custom {
            region: "us-east-1".to_owned(),
            endpoint: self.endpoint_url.to_owned(),
        };
        let credentials = Credentials::new(Some(&self.access_key), Some(&self.secret_key), None, None, None).unwrap();

        let parts: Vec<&str> = folder_path.splitn(2, '/').collect();
        if parts.len() == 2 {
            let bucket_name = parts[0];
            let path = parts[1];
            let bucket = Bucket::new(bucket_name, region.clone(), credentials.clone()).unwrap().with_path_style();
            let response = bucket.put_object(format!("{}{}/", path, folder_name), "".as_bytes()).await;
            match response {
                Ok(_) => {
                    return Ok(());
                }
                Err(error) => {
                    println!("{}", error);
                    return Err(error.to_string());
                }
            }
        }
        Ok(())
    }

    pub async fn delete_object(&self, obj_path: &str) -> Result<(), String> {
        let region = Region::Custom {
            region: "us-east-1".to_owned(),
            endpoint: self.endpoint_url.to_owned(),
        };
        let credentials = Credentials::new(Some(&self.access_key), Some(&self.secret_key), None, None, None).unwrap();
        let parts: Vec<&str> = obj_path.splitn(2, '/').collect();
        if parts.len() == 2 {
            let bucket_name = parts[0];
            let path = parts[1];
            let bucket = Bucket::new(bucket_name, region.clone(), credentials.clone()).unwrap().with_path_style();
            let response = bucket.delete_object(path).await;
            match response {
                Ok(_) => {
                    return Ok(());
                }
                Err(error) => {
                    println!("{}", error);
                    return Err(error.to_string());
                }
            }
        }
        Ok(())
    }

    pub async fn upload_file(&self, folder_path: &str, file_path: &str) -> Result<(), String> {
        let region = Region::Custom {
            region: "us-east-1".to_owned(),
            endpoint: self.endpoint_url.to_owned(),
        };
        let credentials = Credentials::new(Some(&self.access_key), Some(&self.secret_key), None, None, None).unwrap();
        let parts: Vec<&str> = folder_path.splitn(2, '/').collect();
        if parts.len() == 2 {
            let bucket_name = parts[0];
            let path = parts[1];

            let file_path_obj = Path::new(file_path);
            if let Some(file_name) = file_path_obj.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    let file_upload_path = format!("{}/{}", path, file_name_str);
                    let bucket = Bucket::new(bucket_name, region.clone(), credentials.clone()).unwrap().with_path_style();

                    let file_result = File::open(file_path_obj).await;
                    
                    match file_result {
                        Ok(mut file) => {
                            let response = bucket.put_object_stream(&mut file, file_upload_path).await;
                            match response {
                                Ok(_) => {
                                    return Ok(());
                                }
                                Err(error) => {
                                    println!("{}", error);
                                    return Err(error.to_string());
                                }
                            }
                        }
                        Err(error) => {
                            return Err(error.to_string()); 
                        }
                    }
                   
                } else {
                    return Err("File name is not valid UTF-8".to_string());
                }
            } else {
                return Err("No file name found in the path".to_string());
            }
        }
        Ok(())
    }

    pub async fn download_file(&self, file_path: &str, target_path: &str) -> Result<(), String> {
        let region = Region::Custom {
            region: "us-east-1".to_owned(),
            endpoint: self.endpoint_url.to_owned(),
        };
        let credentials = Credentials::new(Some(&self.access_key), Some(&self.secret_key), None, None, None).unwrap();
        let parts: Vec<&str> = file_path.split("/").collect();
        if parts.len() >= 2 {
            let bucket_name = parts[0];
            let file_name = parts[parts.len() - 1];
            let source_path = parts[1..].join("/");

            let download_path = format!("{}/{}", target_path, file_name);
            let async_output_file_result = tokio::fs::File::create(download_path).await;
            match async_output_file_result {
                Ok(mut async_output_file) => {
                    let bucket = Bucket::new(bucket_name, region.clone(), credentials.clone()).unwrap().with_path_style();
                    let result = bucket.get_object_to_writer(source_path, &mut async_output_file).await;
                    match result {
                        Ok(status_code) => {
                            println!("status code: {}", status_code);
                            return Ok(());
                        }
                        Err(error) => {
                            return Err(error.to_string());
                        }
                    }
                }
                Err(error) => {
                    return Err(error.to_string());
                }
            }
        }
        Ok(())
    }
    
}