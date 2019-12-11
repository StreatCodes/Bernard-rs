use bernard::check::{CheckHealthResponse, CheckSystemInfoResponse, Request, Response};
use sysinfo::{SystemExt, RefreshKind, ProcessorExt, DiskExt, DiskType, NetworkExt};
use std::convert::From;
use bernard::check::{Disk, Memory, Cpu, Network};

pub async fn request_handler(req: Request) -> Option<Response> {
    let res = match req {
        Request::CheckSystemInfo(system_info_req) => {
            let refresh = RefreshKind::new()
                .with_disks()
                .with_disk_list()
                .with_network()
                .with_memory()
                .with_cpu();

            let system = sysinfo::System::new_with_specifics(refresh);
            
            let mut cpus: Vec<Cpu> = Vec::new();
            // Then let's print the temperature of the different components:
            for processor in system.get_processor_list() {
                cpus.push(Cpu{
                    name: processor.get_name().to_string(),
                    usage: processor.get_cpu_usage()
                })
            }
            
            let mut disks: Vec<Disk> = Vec::new();
            // And then all disks' information:
            for d in system.get_disks() {
                let t = match d.get_type() {
                    DiskType::HDD => String::from("HDD"),
                    DiskType::SSD => String::from("SSD"),
                    DiskType::Unknown(t) => format!("Unknown({})", t)
                };
        
                let disk = Disk {
                    physical_type: t,
                    name: d.get_name().to_str().unwrap().to_string(),
                    fs_type: String::from_utf8(Vec::from(d.get_file_system())).unwrap(),
                    mount_point: d.get_mount_point().to_str().unwrap().to_string(),
                    total_space: d.get_total_space(),
                    available_space: d.get_available_space(),
                };

                disks.push(disk);
            }

            let net = system.get_network();
            let network = Network {
                incoming: net.get_income(),
                outgoing: net.get_outcome()
            };
            
            let uptime = system.get_uptime();
            
            // And finally the RAM and SWAP information:
            let memory = Memory {
                totalMemory: system.get_total_memory(),
                usedMemory: system.get_used_memory(),
                totalSwap: system.get_total_swap(),
                usedSwap: system.get_used_swap()
            };

            Response::CheckSystemInfo(CheckSystemInfoResponse {
                uptime: uptime,
                cpus: cpus,
                disks: disks,
                net: network,
                memory: memory
            })
        }
        Request::CheckHealth(health_req) => Response::CheckHealth(CheckHealthResponse {}),
    };
    Some(res)
}
