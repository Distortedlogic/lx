# Apache OpenDAL — S3/MinIO Reference for Rust

**Crate**: `opendal` (Apache OpenDAL)
**Repository**: https://github.com/apache/opendal
**Reference submodule**: `reference/opendal/`
**Why this crate**: Apache Foundation project, 4k+ stars, 60+ storage backends, async-first, best-in-class Rust ergonomics. Most well-established and actively maintained S3-compatible object storage crate in the Rust ecosystem.

## Core API: `Operator`

The `Operator` struct is the main entry point. All operations are async.

### Construction (MinIO)

```rust
use opendal::{Operator, services::S3};

let op = Operator::new(S3::default()
    .endpoint("http://127.0.0.1:9000")     // MinIO endpoint
    .region("auto")                         // MinIO doesn't care about region
    .bucket("master-archive")               // Bucket name
    .access_key_id("minioadmin")            // MinIO credentials
    .secret_access_key("minioadmin")
    .disable_config_load()                  // Skip AWS config files
    .disable_ec2_metadata()                 // Not needed for MinIO
)?.finish();
```

### CRUD Operations

| Operation       | Method                  | Signature                                                                      |
| --------------- | ----------------------- | ------------------------------------------------------------------------------ |
| Write bytes     | `op.write(path, bytes)` | `async fn write(&self, path: &str, bs: impl Into<Buffer>) -> Result<Metadata>` |
| Read bytes      | `op.read(path)`         | `async fn read(&self, path: &str) -> Result<Buffer>`                           |
| Delete          | `op.delete(path)`       | `async fn delete(&self, path: &str) -> Result<()>`                             |
| List            | `op.list(path)`         | `async fn list(&self, path: &str) -> Result<Vec<Entry>>`                       |
| Stat            | `op.stat(path)`         | `async fn stat(&self, path: &str) -> Result<Metadata>`                         |
| Exists          | `op.exists(path)`       | `async fn exists(&self, path: &str) -> Result<bool>`                           |
| Batch delete    | `op.delete_iter(iter)`  | `async fn delete_iter<I>(&self, iter: I) -> Result<()>`                        |
| Streaming write | `op.writer(path)`       | `async fn writer(&self, path: &str) -> Result<Writer>`                         |
| Streaming read  | `op.reader(path)`       | `async fn reader(&self, path: &str) -> Result<Reader>`                         |
| Copy            | `op.copy(from, to)`     | `async fn copy(&self, from: &str, to: &str) -> Result<()>`                     |

### Builder Pattern for Options

```rust
// Write with metadata
op.write_with("data.bin")
    .content_type("application/octet-stream")
    .user_metadata("fingerprint", "abc123")
    .await?;

// Read with range
op.read_with("data.bin")
    .range(0..1024*1024)
    .chunk(64 * 1024)
    .concurrent(4)
    .await?;

// List with options
op.list_with("individuals/")
    .recursive(true)
    .await?;
```

### Streaming Write (for large objects)

```rust
let mut writer = op.writer("large_file.bin").await?;
writer.write(chunk1).await?;
writer.write(chunk2).await?;
let metadata = writer.close().await?;  // Finalizes multipart upload
```

### Error Handling

```rust
use opendal::ErrorKind;

match op.read("file.bin").await {
    Ok(buffer) => { /* use buffer */ },
    Err(err) => match err.kind() {
        ErrorKind::NotFound => { /* 404 */ },
        ErrorKind::PermissionDenied => { /* 403 */ },
        ErrorKind::RateLimited => { /* 429, retryable */ },
        _ => { /* other error */ },
    }
}
```

**Error properties**: `err.is_temporary()` (retryable), `err.is_permanent()` (config/auth error).

## S3Config Full Fields

```rust
pub struct S3Config {
    pub root: Option<String>,                    // Base directory prefix
    pub bucket: String,                          // REQUIRED
    pub endpoint: Option<String>,                // Custom endpoint (required for MinIO)
    pub region: Option<String>,                  // AWS region or "auto"
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,           // STS temporary credentials
    pub role_arn: Option<String>,                // IAM assume role
    pub allow_anonymous: bool,                   // Public buckets
    pub enable_virtual_host_style: bool,         // bucket.endpoint vs endpoint/bucket
    pub delete_max_size: Option<usize>,          // Batch delete limit (default 1000)
    pub checksum_algorithm: Option<String>,      // "crc32c" or "md5"
    pub default_storage_class: Option<String>,   // STANDARD, GLACIER, etc.
    pub server_side_encryption: Option<String>,  // "AES256" or "aws:kms"
    pub enable_versioning: bool,
    pub disable_config_load: bool,               // Skip ~/.aws/config
    pub disable_ec2_metadata: bool,              // Skip IMDSv2
    // ... additional SSE and compatibility fields
}
```

## Architecture for Master Archive Integration

### Recommended Bucket/Prefix Structure

```
master-archive/                    (bucket)
├── individuals/                   (prefix for individual strategy files)
│   ├── {fingerprint}.bin.zst     (bincode+zstd compressed ArchivedIndividual)
│   └── ...
├── manifests/                     (prefix for manifest snapshots)
│   ├── latest.bin.zst            (current manifest)
│   └── {timestamp}.bin.zst       (historical manifests for recovery)
└── metadata/                      (prefix for archive metadata)
    └── config.json               (archive configuration)
```

### Write Pattern (replacing filesystem writes)

```rust
// Current: save_bincode_zstd writes to local file
// New: encode to buffer, then upload to MinIO

let mut buf = Vec::new();
encode_bincode_zstd_to_writer(&mut buf, &archived_individual, 3)?;
op.write(&format!("individuals/{}.bin.zst", fingerprint), buf).await?;
```

### Read Pattern (replacing filesystem reads)

```rust
// Current: load_bincode_zstd reads from local file
// New: download from MinIO, then decode

let bytes = op.read(&format!("individuals/{}.bin.zst", fingerprint)).await?;
let individual: ArchivedIndividual = decode_bincode_zstd_from_reader(
    std::io::Cursor::new(bytes.to_vec()),
    MAX_INDIVIDUAL_BYTES,
)?;
```

## Key Advantages Over Filesystem

1. **Decoupled from local disk**: Archive persists independently of the compute instance
2. **Concurrent access**: Multiple evolution runs can read the same archive safely
3. **Scalability**: MinIO handles millions of objects with consistent performance
4. **Versioning**: Built-in object versioning for manifest recovery
5. **Lifecycle policies**: Automatic tiering/expiry of old objects
6. **HTTP API**: Archive browsable via MinIO Console web UI

## Source Files in Reference Submodule

- `reference/opendal/core/core/src/types/operator/operator.rs` — Main Operator API
- `reference/opendal/core/services/s3/src/backend.rs` — S3Builder & S3Backend
- `reference/opendal/core/services/s3/src/config.rs` — S3Config struct
- `reference/opendal/core/services/s3/src/error.rs` — S3 error mapping
- `reference/opendal/core/services/s3/src/docs.md` — Usage examples
- `reference/opendal/core/services/s3/src/compatible_services.md` — MinIO compatibility notes
