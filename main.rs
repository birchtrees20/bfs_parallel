use rayon::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};

type Maze = Vec<Vec<char>>;

#[derive(Debug, Clone)]
struct SharedState {
    visited: HashSet<(usize, usize)>,
    queue: VecDeque<(usize, usize)>,
    parents: Vec<Vec<Option<(usize, usize)>>>,
    //goal_reached: bool //goal_reached flag
}

impl SharedState {
    fn new(rows: usize, cols: usize) -> Self {
        SharedState {
            visited: HashSet::new(),
            queue: VecDeque::new(),
            parents: vec![vec![None; cols]; rows],
        }
    }
}

fn bfs_worker(
    maze: Arc<Maze>,
    state: Arc<(Mutex<SharedState>, AtomicBool)>,
    start: (usize, usize),
    goal: (usize, usize),
    id: usize,
) -> Option<Vec<(usize, usize)>> {
    loop {
        let (state_lock, goal_reached) = &*state;
        let cell = {
            let mut state = state_lock.lock().unwrap();
            if goal_reached.load(Ordering::Relaxed) {
                return None; // Stop processing if the goal is reached
            }
            state.queue.pop_front()
        };

        match cell {
            Some(current_cell) => {
                if current_cell == goal {
                    println!("Thread {}: Goal reached!", id);
                    goal_reached.store(true, Ordering::Relaxed);
                    return Some(retrieve_shortest_path(&state_lock, start, goal));
                }

                if let Some(neighbors) = get_neighbors(&maze, current_cell, goal_reached) {
                    for neighbor in neighbors {
                        let mut state = state_lock.lock().unwrap();
                        if state.visited.insert(neighbor) {
                            state.queue.push_back(neighbor);
                            state.parents[neighbor.0][neighbor.1] = Some(current_cell);
                        }
                    }
                }

                println!("Thread {}: Processing cell {:?} ", id, current_cell);
            }
            None => break,
        }
    }

    None
}

fn get_neighbors(
    maze: &Maze,
    cell: (usize, usize),
    goal_reached: &AtomicBool,
) -> Option<Vec<(usize, usize)>> {
    // Check if the goal is reached, and if so, return early
    if goal_reached.load(Ordering::Relaxed) {
        return None;
    }

    let (row, col) = cell;
    let mut neighbors = Vec::new();

    for &(dr, dc) in &[(1, 0), (0, 1), (-1, 0), (0, -1)] {
        let new_row = row.wrapping_add(dr as usize);
        let new_col = col.wrapping_add(dc as usize);

        if new_row < maze.len() && new_col < maze[0].len() && maze[new_row][new_col] != '#' {
            neighbors.push((new_row, new_col));
        }
    }

    if neighbors.is_empty() {
        None
    } else {
        Some(neighbors)
    }
}

fn retrieve_shortest_path(
    state: &Mutex<SharedState>,
    start: (usize, usize),
    goal: (usize, usize),
) -> Vec<(usize, usize)> {
    let state = state.lock().unwrap();
    let mut current = goal;
    let mut path = vec![current];

    while current != start {
        let parent = state.parents[current.0][current.1];
        if let Some(p) = parent {
            path.push(p);
            current = p;
        } else {
            break;
        }
    }

    path.iter().rev().cloned().collect()
}

fn parallel_bfs(maze: &Maze, start: (usize, usize), goal: (usize, usize)) -> Option<Vec<(usize, usize)>> {
    let num_threads = 6;
    let maze = Arc::new(maze.clone());
    let state = Arc::new((
        Mutex::new(SharedState::new(maze.len(), maze[0].len())),
        AtomicBool::new(false),
    ));

    state.0.lock().unwrap().queue.push_back(start);

    let mut handles = vec![];

    for i in 0..num_threads {
        let maze_clone = Arc::clone(&maze);
        let state_clone = Arc::clone(&state);

        let handle = thread::spawn(move || {
            bfs_worker(maze_clone, state_clone, start, goal, i)
        });

        handles.push(handle);
    }

    for handle in handles {
        if let Some(path) = handle.join().unwrap() {
            return Some(path);
        }
    }

    None
}

fn main() {
    println!("Hello, world!");
    let maze: Maze = vec![
        vec!['.', '#', '.', '.', '.', '.', '.'],
        vec!['.', '#', '.', '#', '#', '#', '.'],
        vec!['.', '#', '.', '.', '.', '#', '.'],
        vec!['.', '#', '#', '#', '.', '#', '.'],
        vec!['.', '.', '.', '.', '.', '#', '.'],
        vec!['#', '#', '#', '#', '.', '#', '.'],
        vec!['.', '.', '.', '.', '.', '#', '.'],
    ];
    
    let start = (0, 0);
    let goal = (6, 6);

    if let Some(shortest_path) = parallel_bfs(&maze, start, goal) {
        println!("Shortest Path is: {:?}", shortest_path);
    } else {
        println!("No path found");
    }
}

/*
//make other threads stop once goal reached
In parallel: convert path to maze parallel using par iter rayon
making a solution vector using parallel

*/
