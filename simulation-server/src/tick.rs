fn handle_all_movements(lock: &RwLock<GameState>) {
    let time_instance = Instant::now();

    let mut time_step: f32 = 1.;
    let mut count = 1.;

    //TOOD: Get start time

    loop {
        //============ METHOD 02 ===============
        // Get elapsed time and check if it is less than the a second
        if time_instance.elapsed().as_secs_f32() < time_step {
            //Condition to simulation x number of times; 50 in this case
            if count <= SERVER_TICK {
                let time_before_simulation = Instant::now();

                // Simulation Logic
                let mut w = lock.write().unwrap();
                let game_state = &mut *w;
                handle_bead_movement(game_state);
                handle_predator_movement_system(game_state);

                let time_after_simulation = time_before_simulation.elapsed().as_secs_f32();

                // Sleep thread for time leftover from x ms - which is the maximum time per iteration
                thread::sleep(Duration::from_millis(15 - time_after_simulation as u64));

                println!(
                    "This runs: {:?} with count {}",
                    time_instance.elapsed(),
                    count
                );

                count += 1.;
            } else {
                // Sleep thread for time leftover from x ms - which is the maximum time per iteration
                // In this case it technically should be 20ms - but since there's some code after the thread sleep 15ms works best

                if time_instance.elapsed().as_secs_f32() < time_step {
                    thread::sleep(Duration::from_secs_f32(
                        time_step - time_instance.elapsed().as_secs_f32(),
                    ));
                }
                println!("Time : {:?}", time_instance.elapsed().as_secs_f32());
            }
        } else {
            //Increase time step and reset the count
            time_step += 1.;
            count = 1.;
        }

        //TODO : End time
        // 20 - elapsed_time : sleep time
    }
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8888").expect("Could not bind socket");

    // let (tx, rx) = spmc::channel();
    let mut handles = vec![];

    let mut beads: EntityState = EntityState {
        entity_atrib: HashMap::new(),
    };
    let mut predators: PredatorState = PredatorState {
        predator_atrib: HashMap::new(),
    };

    spawn_entities(&mut beads, &mut predators);

    let game_state = GameState {
        entity_state: beads,
        predator_state: predators,
    };

    let g_state = Arc::new(RwLock::new(game_state));
    let game_state_info = Arc::new(Rw::new(game_state.clone()));
    
    let game_info = Arc::clone(&game_state_info);
    let g_state_2 = Arc::clone(g_state);

    handles.push(thread::spawn(move || {
        handle_all_movements(&game_state_info);
    }));

    handles.psuh(thread::spawn(move || {
        simulaiton_tick(&game_info, &g_state);  
    }));

    let c_lock = Arc::clone(&lock);
    handles.push(thread::spawn(move || {
        handle_connection(socket, &g_state_2)
        .unwrap_or_else(|error| eprintln!("{:?}", error));
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}

fn simulaiton_tick(info: &RwLock<GameState>, state: &RwLock<GameState>) {

    let mut time_step: f32 = 0.03333; // 1/30 aproximated upto 5 decimal places
    let mut tick = 1;
    
    loop {
        while (tick != 30) {

            let time_instance = Instant::now();
                
                while (time_instance.elapsed().as_secs_f32() <= time_step) {

                    // let mut w = lock.write().unwrap();
                    // let game_state = &mut *w;
                    // handle_bead_movement(game_state);
                    // handle_predator_movement_system(game_state);
                    state = info;
                    thread::sleep(Duration::from_secs((time_step  - time_instance.elapsed().as_secs_f32()) as u64 ));
                }
            tick += 1;
            println!("{}", tick);
        }
        tick = 1;
    }
}