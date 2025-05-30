# hv_sock

`hv_sock` is a Rust library providing a cross-platform API for Hyper-V sockets (AF_HYPERV on Windows, AF_VSOCK on Linux). It allows communication between a host machine and its virtual machines, or between virtual machines.

## Features

*   **Cross-Platform:** Works on both Windows (with AF_HYPERV) and Linux (with AF_VSOCK).
*   **Socket API:** Familiar socket operations for listeners and streams:
    *   `HyperVSocketListener`: Bind to a local address and listen for incoming connections.
    *   `HyperVSocketStream`: Connect to a remote Hyper-V socket and exchange data.
*   **Typed Addresses:** `HyperVSocketAddr` for platform-specific Hyper-V socket addressing.
*   **Windows Host Registry (Optional Feature):**
    *   The `host-registry` feature (Windows-only) provides utilities to interact with the Windows Host Communication Services registry for service discovery.
    *   Includes types like `HostRegistry`, `Service`, and `ServiceUuid`, and constants for well-known Hyper-V GUIDs (e.g., `WILDCARD`, `LOOPBACK`).

## Supported Platforms

*   **Windows:** Requires Windows 10 / Windows Server 2016 or later.
*   **Linux:** Requires Linux kernel with VSOCK support (typically 4.8+).

## Getting Started

Add `hv_sock` to your `Cargo.toml`:

```toml
[dependencies]
hv_sock = "0.1.0" # Replace with the latest version from Crates.io

# For Windows, uuid is a dependency of hv_sock.
# If you use Uuid in your own code (e.g. for service IDs), ensure version compatibility.
# hv_sock itself depends on uuid = "1.10.0".
# You might add it to your [dependencies] or [target.'cfg(windows)'.dependencies]
# if your application code directly uses it. For example:
#
# [dependencies]
# uuid = { version = "1.10.0", features = ["v4"] }
```

To enable the Windows-specific `host-registry` feature:

```toml
[dependencies]
hv_sock = { version = "0.1.0", features = ["host-registry"] }
```

## Usage

### Creating Socket Addresses (`HyperVSocketAddr`)

Socket addresses are platform-specific:

*   **On Windows:** A `HyperVSocketAddr` is defined by a VM ID and a Service ID (both UUIDs).
    ```rust
    use hv_sock::HyperVSocketAddr;
    use uuid::Uuid;

    // Example: Listening for any VM (VMID_WILDCARD) on a specific service ID
    let my_service_id = Uuid::new_v4(); // Your unique service ID

    // VMID_WILDCARD is typically Uuid::nil().
    // If using the `host-registry` feature, you can use `hv_sock::host_registry::WILDCARD`.
    let listen_addr = HyperVSocketAddr::new(Uuid::nil(), my_service_id);

    // Example: Connecting to a specific VM on a specific service ID
    // let vm_id = Uuid::parse_str("your-vm-guid-here").unwrap();
    // let connect_addr = HyperVSocketAddr::new(vm_id, my_service_id);

    // For connecting to the host (loopback):
    // HV_GUID_LOOPBACK: Uuid::from_fields(0x40e0f4a4, 0x9ac6, 0xa74e, &[0x8b,0x96,0x01,0xfa,0x50,0xc1,0x7a,0x86])
    // If using `host-registry` feature, `hv_sock::host_registry::LOOPBACK` can be used.
    ```

*   **On Linux:** A `HyperVSocketAddr` is defined by a Context ID (CID) and a Port.
    The current version of `HyperVSocketAddr::new(port: u32)` implicitly uses `VMADDR_CID_HOST` for the CID. This is suitable for services running on the host listening for VM connections, or for host-to-host communication.
    ```rust
    use hv_sock::HyperVSocketAddr;

    // Example: Listening on a specific port (CID is implicitly HOST)
    let port = 12345;
    let addr = HyperVSocketAddr::new(port);
    ```

### Example: Simple Echo Server and Client

This example demonstrates a basic echo server and client. In a real application, the server and client would typically run in different processes or VMs.

**Add `uuid` to your `Cargo.toml` if you are on Windows and following this example:**
```toml
[dependencies]
hv_sock = "0.1.0"
uuid = { version = "1.10.0", features = ["v4"] } # Needed for Uuid::new_v4, Uuid::parse_str etc.
```

```rust
use hv_sock::{HyperVSocketListener, HyperVSocketStream, HyperVSocketAddr};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

// Platform-specific imports and constants
#[cfg(windows)]
use uuid::Uuid;

// Define a shared service identifier
#[cfg(windows)]
const SERVICE_ID_STR: &str = "c2a4832c-f2a7-4faf-b513-b9a0a7a9d177"; // EXAMPLE GUID - REPLACE THIS

#[cfg(target_os = "linux")]
const SERVICE_PORT: u32 = 12345; // Example port for Linux

#[cfg(windows)]
fn get_server_addr() -> HyperVSocketAddr {
    let service_id = Uuid::parse_str(SERVICE_ID_STR)
        .expect("Failed to parse SERVICE_ID_STR. Please use a valid GUID.");
    // Listen on WILDCARD VMID (Uuid::nil()) and the specific service_id
    HyperVSocketAddr::new(Uuid::nil(), service_id)
}

#[cfg(windows)]
fn get_client_connect_addr() -> HyperVSocketAddr {
    let service_id = Uuid::parse_str(SERVICE_ID_STR)
        .expect("Failed to parse SERVICE_ID_STR. Please use a valid GUID.");
    // Connect to LOOPBACK VMID and the specific service_id
    // HV_GUID_LOOPBACK is Uuid::from_fields(0x40e0f4a4, 0x9ac6, 0xa74e, &[0x8b,0x96,0x01,0xfa,0x50,0xc1,0x7a,0x86])
    // This constant can be obtained from `hv_sock::host_registry::LOOPBACK` if 'host-registry' feature is enabled.
    const LOOPBACK_VM_ID: Uuid = Uuid::from_fields(0x40e0f4a4, 0x9ac6, 0xa74e, &[0x8b,0x96,0x01,0xfa,0x50,0xc1,0x7a,0x86]);
    HyperVSocketAddr::new(LOOPBACK_VM_ID, service_id)
}

#[cfg(target_os = "linux")]
fn get_server_addr() -> HyperVSocketAddr {
    // Listen on SERVICE_PORT, CID is implicitly VMADDR_CID_HOST
    HyperVSocketAddr::new(SERVICE_PORT)
}

#[cfg(target_os = "linux")]
fn get_client_connect_addr() -> HyperVSocketAddr {
    // Connect to SERVICE_PORT, CID is implicitly VMADDR_CID_HOST
    HyperVSocketAddr::new(SERVICE_PORT)
}

fn run_server() -> std::io::Result<()> {
    let listen_addr = get_server_addr();
    let listener = HyperVSocketListener::bind(&listen_addr)?;
    println!("Server listening on: {:?}", listener.local_addr().unwrap_or(listen_addr));

    // Accept one connection
    match listener.accept() {
        Ok((mut stream, client_addr)) => {
            println!("Server accepted connection from: {:?}", client_addr);
            let mut buffer = [0u8; 1024];
            let bytes_read = stream.read(&mut buffer)?;
            if bytes_read > 0 {
                let received_msg = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Server received: '{}'", received_msg);
                stream.write_all(b"Echo from server: ")?;
                stream.write_all(&buffer[..bytes_read])?;
                println!("Server sent echo.");
            }
        }
        Err(e) => {
            eprintln!("Server accept error: {}", e);
        }
    }
    Ok(())
}

fn run_client() -> std::io::Result<()> {
    let connect_addr = get_client_connect_addr();
    println!("Client attempting to connect to: {:?}", connect_addr);
    let mut stream = HyperVSocketStream::connect(&connect_addr)?;
    println!("Client connected to server (peer: {:?}, local: {:?})", stream.peer_addr().unwrap_or(connect_addr), stream.local_addr().ok());

    let message = "Hello from client!";
    stream.write_all(message.as_bytes())?;
    println!("Client sent: '{}'", message);

    let mut buffer = [0u8; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read > 0 {
        let received_msg = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Client received: '{}'", received_msg);
    } else {
        println!("Client received no data.");
    }
    Ok(())
}

fn main() {
    // IMPORTANT FOR WINDOWS USERS:
    // Replace SERVICE_ID_STR with your own unique GUID before running.
    // You can generate one using `uuidgen` (Linux/macOS) or `New-Guid` (PowerShell).
    #[cfg(windows)]
    if SERVICE_ID_STR == "c2a4832c-f2a7-4faf-b513-b9a0a7a9d177" { // Default example GUID
        println!("INFO: Using default example SERVICE_ID_STR. Consider replacing it with your own unique GUID for production or testing.");
    }

    let server_thread = thread::spawn(|| {
        println!("Server thread started.");
        if let Err(e) = run_server() {
            eprintln!("Server error: {}", e);
        }
        println!("Server thread finished.");
    });

    // Give the server a moment to start and bind
    thread::sleep(Duration::from_secs(1));

    println!("Client thread starting.");
    if let Err(e) = run_client() {
        eprintln!("Client error: {}", e);
    }
    println!("Client thread finished.");

    server_thread.join().expect("Server thread panicked");
    println!("Example finished.");
}

```

## Building

To build the library:
```bash
cargo build
```

To build with the `host-registry` feature (on Windows):
```bash
cargo build --features host-registry
```

## License

This project is licensed under the MIT license.
