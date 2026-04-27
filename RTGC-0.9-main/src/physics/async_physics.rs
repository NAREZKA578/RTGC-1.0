use crate::physics::{Helicopter, PhysicsWorld, RigidBody};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

/// Сообщение для физического потока
pub enum PhysicsMessage {
    Step { dt: f32, sub_steps: u32 },
    SetBodies(Vec<RigidBody>),
    SetHelicopter(Helicopter),
    GetBodies,
    GetHelicopter,
    Shutdown,
}

/// Ответ от физического потока
pub enum PhysicsResponse {
    Bodies(Vec<RigidBody>),
    Helicopter(Helicopter),
    StepComplete,
    ShutdownComplete,
}

/// Асинхронный физический движок с double buffering и поддержкой вертолётов
pub struct AsyncPhysicsEngine {
    sender: Sender<PhysicsMessage>,
    receiver: Receiver<PhysicsResponse>,
    running: Arc<AtomicBool>,
    local_bodies: Vec<RigidBody>,
    pending_bodies: Option<Vec<RigidBody>>,
    local_helicopter: Option<Helicopter>,
    pending_helicopter: Option<Helicopter>,
}

impl AsyncPhysicsEngine {
    pub fn new() -> Self {
        let (msg_sender, msg_receiver) = channel();
        let (resp_sender, resp_receiver) = channel();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        // Запуск физического потока
        thread::spawn(move || {
            let mut bodies: Vec<RigidBody> = Vec::new();
            let mut helicopter: Option<Helicopter> = None;

            while running_clone.load(Ordering::Relaxed) {
                match msg_receiver.recv() {
                    Ok(PhysicsMessage::Step { dt, sub_steps: _ }) => {
                        // A4: Использовать PhysicsWorld вместо заглушки
                        // Создать временный PhysicsWorld, передать тела, вызвать step()
                        let mut world = PhysicsWorld::new();
                        for body in bodies.drain(..) {
                            world.add_body(body);
                        }

                        // Обновление вертолёта если есть
                        if let Some(ref mut heli) = helicopter {
                            // Вертолёт использует свою собственную систему сил
                            // gravity уже учтён в его update() через internal state
                            heli.update(dt);

                            // Синхронизировать позицию вертолёта с physics world
                            // (если вертолёт должен взаимодействовать с другими телами)
                        }

                        world.step(dt); // ← вся физика включая коллизии

                        // Забрать обновлённые тела обратно
                        bodies = world.rigid_bodies.iter().cloned().collect();

                        let _ = resp_sender.send(PhysicsResponse::StepComplete);
                    }

                    Ok(PhysicsMessage::SetBodies(new_bodies)) => {
                        bodies = new_bodies;
                        let _ = resp_sender.send(PhysicsResponse::StepComplete);
                    }

                    Ok(PhysicsMessage::SetHelicopter(heli)) => {
                        helicopter = Some(heli);
                        let _ = resp_sender.send(PhysicsResponse::StepComplete);
                    }

                    Ok(PhysicsMessage::GetBodies) => {
                        let _ = resp_sender.send(PhysicsResponse::Bodies(bodies.clone()));
                    }

                    Ok(PhysicsMessage::GetHelicopter) => {
                        let response = if let Some(ref heli) = helicopter {
                            PhysicsResponse::Helicopter(heli.clone())
                        } else {
                            PhysicsResponse::StepComplete
                        };
                        let _ = resp_sender.send(response);
                    }

                    Ok(PhysicsMessage::Shutdown) => {
                        let _ = resp_sender.send(PhysicsResponse::ShutdownComplete);
                        break;
                    }

                    Err(_) => break,
                }
            }
        });

        Self {
            sender: msg_sender,
            receiver: resp_receiver,
            running,
            local_bodies: Vec::new(),
            pending_bodies: None,
            local_helicopter: None,
            pending_helicopter: None,
        }
    }

    /// Установить тела для симуляции (double buffer)
    pub fn set_bodies(&mut self, bodies: Vec<RigidBody>) {
        self.pending_bodies = Some(bodies);
    }

    /// Установить вертолёт для симуляции
    pub fn set_helicopter(&mut self, helicopter: Helicopter) {
        self.pending_helicopter = Some(helicopter);
    }

    /// Синхронизировать локальные данные с потоком
    /// 
    /// # Важное замечание по использованию
    /// 
    /// Этот метод читает ответы из канала `receiver`. Не вызывайте `wait_for_step()`
    /// сразу после `sync()` без промежуточного `step()`, иначе может произойти deadlock:
    /// 
    /// ```no_run
    /// // НЕПРАВИЛЬНО - может вызвать deadlock:
    /// engine.step(dt, sub_steps);
    /// engine.sync();        // Поглощает ответ StepComplete
    /// engine.wait_for_step(); // Блокируется навсегда, ожидая второй ответ
    /// 
    /// // ПРАВИЛЬНО:
    /// engine.step(dt, sub_steps);
    /// engine.wait_for_step(); // Ждём завершения шага
    /// engine.sync();          // Синхронизируем результаты
    /// ```
    pub fn sync(&mut self) {
        // Синхронизация тел
        if let Some(pending) = self.pending_bodies.take() {
            self.local_bodies = pending.clone();
            let _ = self.sender.send(PhysicsMessage::SetBodies(pending));
            let _ = self.receiver.recv();
        } else {
            match self.receiver.try_recv() {
                Ok(PhysicsResponse::Bodies(bodies)) => {
                    self.local_bodies = bodies;
                }
                _ => {}
            }
        }

        // Синхронизация вертолёта
        if let Some(pending) = self.pending_helicopter.take() {
            self.local_helicopter = Some(pending.clone());
            let _ = self.sender.send(PhysicsMessage::SetHelicopter(pending));
            let _ = self.receiver.recv();
        } else {
            match self.receiver.try_recv() {
                Ok(PhysicsResponse::Helicopter(heli)) => {
                    self.local_helicopter = Some(heli);
                }
                _ => {}
            }
        }
    }

    /// Шаг симуляции
    pub fn step(&mut self, dt: f32, sub_steps: u32) {
        let _ = self.sender.send(PhysicsMessage::Step { dt, sub_steps });
    }

    /// Ожидать завершения шага
    pub fn wait_for_step(&self) {
        let _ = self.receiver.recv();
    }

    /// Получить текущие тела
    pub fn get_bodies(&self) -> &[RigidBody] {
        &self.local_bodies
    }

    /// Получить вертолёт
    pub fn get_helicopter(&self) -> Option<&Helicopter> {
        self.local_helicopter.as_ref()
    }

    /// Получить состояние вертолёта
    pub fn get_helicopter_state(&self) -> Option<crate::physics::HelicopterState> {
        self.local_helicopter.as_ref().map(|h| h.get_state())
    }

    /// Отправить запрос на получение тел из потока
    pub fn request_bodies(&self) {
        let _ = self.sender.send(PhysicsMessage::GetBodies);
    }

    /// Отправить запрос на получение вертолёта
    pub fn request_helicopter(&self) {
        let _ = self.sender.send(PhysicsMessage::GetHelicopter);
    }

    /// Остановить движок
    pub fn shutdown(self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.sender.send(PhysicsMessage::Shutdown);
        let _ = self.receiver.recv();
    }
}

impl Default for AsyncPhysicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_async_physics_creation() {
        let engine = AsyncPhysicsEngine::new();
        assert!(engine.running.load(Ordering::Relaxed));
    }

    #[test]
    fn test_async_physics_step() {
        let mut engine = AsyncPhysicsEngine::new();

        let sphere = RigidBody::new_sphere(Vector3::new(0.0, 10.0, 0.0), 1.0, 0.5);

        engine.set_bodies(vec![sphere]);
        engine.sync();

        engine.step(0.016, 4);
        engine.wait_for_step();

        engine.request_bodies();
        engine.sync();

        let bodies = engine.get_bodies();
        assert_eq!(bodies.len(), 1);
        // Тело должно упасть под действием гравитации
        assert!(bodies[0].position.y < 10.0);
    }

    #[test]
    fn test_helicopter_physics() {
        let mut engine = AsyncPhysicsEngine::new();

        let heli = Helicopter::new(Vector3::new(0.0, 10.0, 0.0));
        engine.set_helicopter(heli);
        engine.sync();

        assert!(engine.get_helicopter().is_some());

        // Запуск двигателя и взлёт
        if let Some(ref mut heli) = engine.local_helicopter {
            heli.engine.start_engine();
            heli.controls.collective = 0.6;
            heli.controls.throttle = 0.9;
        }
        engine.sync();

        engine.step(0.016, 4);
        engine.wait_for_step();

        engine.request_helicopter();
        engine.sync();

        // Вертолёт должен набирать обороты ротора
        if let Some(heli) = engine.get_helicopter() {
            assert!(heli.main_rotor.current_rpm > 0.0);
        }
    }
}
