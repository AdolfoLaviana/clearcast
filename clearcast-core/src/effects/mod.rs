//! Módulo para efectos de audio personalizados
//!
//! Este módulo proporciona una interfaz unificada para crear y gestionar efectos
//! de audio en ClearCast. Los efectos pueden ser encadenados para crear cadenas
//! de procesamiento complejas.
//!
//! # Características
//! - Interfaz unificada para todos los efectos
//! - Fácil de extender con nuevos efectos
//! - Seguro para uso en entornos multihilo
//! - Bajo overhead de rendimiento
//!
//! # Uso Básico
//! ```rust
//! use clearcast_core::effects::{Delay, AudioEffect};
//! use std::sync::{Arc, Mutex};
//!
//! // Crear un efecto de delay
//! let mut delay = Delay::new(300.0, 0.5, 0.3, 0.7, 44100);
//!
//! // Procesar una muestra
//! let sample = 0.5;
//! let processed = delay.process_sample(sample);
//!
//! // Para uso seguro en hilos
//! let thread_safe_delay = delay.boxed();
//! ```

mod delay;
pub use delay::Delay;

/// Interfaz base para todos los efectos de audio
///
/// Este trait define la interfaz que deben implementar todos los efectos de audio
/// en ClearCast. Los efectos pueden ser procesadores de señal simples o complejos,
/// y pueden mantener un estado interno.
///
/// # Requisitos de Seguridad en Hilos
/// - `Send`: Permite transferir la propiedad del efecto entre hilos
/// - `Sync`: Permite acceso compartido al efecto desde múltiples hilos
///
/// # Ejemplo de Implementación
/// ```rust
/// use clearcast_core::effects::AudioEffect;
///
/// struct Gain {
///     amount: f32,
/// }
///
/// impl AudioEffect for Gain {
///     fn process_sample(&mut self, sample: f32) -> f32 {
///         sample * self.amount
///     }
///     
///     fn reset(&mut self) {
///         // Reiniciar estado interno si es necesario
///     }
///     
///     fn name(&self) -> &'static str {
///         "Gain"
///     }
/// }
/// ```
pub trait AudioEffect: Send + Sync {
    /// Procesa una muestra de audio
    fn process_sample(&mut self, sample: f32) -> f32;
    
    /// Procesa un búfer de audio completo
    fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }
    
    /// Reinicia el estado interno del efecto
    fn reset(&mut self);
    
    /// Devuelve el nombre del efecto
    fn name(&self) -> &'static str;
    
    /// Crea una nueva instancia en un Arc<Mutex<Self>> para uso seguro en hilos
    fn boxed(self) -> std::sync::Arc<std::sync::Mutex<Self>> 
    where 
        Self: Sized + 'static 
    {
        std::sync::Arc::new(std::sync::Mutex::new(self))
    }
}
