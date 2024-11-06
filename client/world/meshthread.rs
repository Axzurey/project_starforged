use std::{collections::HashMap, sync::{mpsc::{self, Receiver, Sender}, Arc}, thread};

use shared::world::chunk::Chunk;
use stopwatch::Stopwatch;
use wgpu::util::DeviceExt;

use crate::renderer::renderctx::Renderctx;

use super::mesher::mesh_slice_arrayed;

pub fn spawn_chunk_meshing_worker_thread(
    id: usize,
    send_back: Sender<(usize, i32, i32, u32, ((wgpu::Buffer, wgpu::Buffer, u32), (wgpu::Buffer, wgpu::Buffer, u32)))>
) -> Sender<(i32, i32, u32, HashMap<u32, Arc<Chunk>>, Arc<Renderctx>)> {
    let (send, recv) = mpsc::channel::<(i32, i32, u32, HashMap<u32, Arc<Chunk>>, Arc<Renderctx>)>();
    

    thread::spawn(move || {
        while let Ok((chunk_x, chunk_z, y_slice, mut chunks, ctx)) = recv.recv() {
            let t = Stopwatch::start_new();
            let result = mesh_slice_arrayed(chunk_x, chunk_z, y_slice, &chunks);

            let ((vertices, indices, ilen), (vertices_transparent, indices_transparent, ilen_t, quads)) = (result.0, result.1);

            let vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Chunk Vertex Buffer")),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Chunk Index Buffer")),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            let vertex_buffer_t = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Chunk Vertex Buffer Transparent")),
                contents: bytemuck::cast_slice(&vertices_transparent),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer_t = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Chunk Index Buffer Transparent")),
                contents: bytemuck::cast_slice(&indices_transparent),
                usage: wgpu::BufferUsages::INDEX,
            });


            send_back.send((id, chunk_x, chunk_z, y_slice, ((vertex_buffer, index_buffer, ilen), (vertex_buffer_t, index_buffer_t, ilen_t)))).unwrap();
            
        }
        
    });

    send
}

pub fn spawn_chunk_meshing_loop(
    num_workers: usize
) -> (
    Sender<(i32, i32, u32, HashMap<u32, Arc<Chunk>>, Arc<Renderctx>)>,
    Receiver<(i32, i32, u32, ((wgpu::Buffer, wgpu::Buffer, u32), (wgpu::Buffer, wgpu::Buffer, u32)))>
) {
    //unapologetically stolen from elttob
    let (frommain, frommainrecv) = mpsc::channel();
    let (tomain, tomainrecv) = mpsc::channel();

    let (worker_send_finished_chunks, worker_recv_finished_chunks) = mpsc::channel();
    let (send_idle_worker, recv_idle_worker) = mpsc::channel();

    for id in 0..num_workers {
        send_idle_worker.send(id).unwrap();
    }

    thread::spawn(move || {
        let send_idle_worker = send_idle_worker.clone();
        loop {
            let data: (usize, i32, i32, u32, ((wgpu::Buffer, wgpu::Buffer, u32), (wgpu::Buffer, wgpu::Buffer, u32))) = worker_recv_finished_chunks.recv().unwrap();
            let id = data.0.clone();
            tomain.send((data.1, data.2, data.3, data.4)).unwrap();
            send_idle_worker.send(id).unwrap();
        }
    });

    thread::spawn(move || {
        let mut workers = (0..num_workers).map(|id| {
            spawn_chunk_meshing_worker_thread(id, worker_send_finished_chunks.clone())
        }).collect::<Vec<_>>();

        loop {
            let next_data = frommainrecv.recv().unwrap();
            let next = recv_idle_worker.recv().unwrap();
            let worker = &mut workers[next];
            worker.send(next_data).unwrap();
        }
    });

    (frommain, tomainrecv)
}