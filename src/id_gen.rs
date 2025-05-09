use std::sync::atomic::AtomicUsize;

// Um acumulador global para gerar IDs exclusivos.
// O AtomicUsize garante segurança em acesso concorrente.
static ACCUMULATOR: AtomicUsize = AtomicUsize::new(0);

pub type UniqueId = usize;

// Função para gerar um ID único.
// Utiliza fetch_add com Ordering::Relaxed para desempenho otimizado.
pub fn generate_id() -> UniqueId {
    // Ordering::Relaxed é suficiente aqui, pois não há dependência de ordenação de memória (simples incremento)
    ACCUMULATOR.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}
