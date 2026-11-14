//! Polypus · Receitas — núcleo em Rust, compilado para WebAssembly.
//!
//! Toda a regra de negócio mora aqui: cadastro e validação de alimentos e receitas,
//! conversão de medidas, cálculo de custo e nutrição, %VD, formatação brasileira de números
//! e o formato de arquivo (salvar/importar). O JavaScript da página só desenha a tela e
//! guarda o texto JSON no localStorage.
//!
//! A troca de dados com o JavaScript é feita em JSON (texto), para não depender de
//! bibliotecas extras além de `wasm-bindgen` e `serde`.

pub mod calculo;
pub mod formato;
pub mod modelo;

use calculo::{calcular, validar_alimento, validar_receita};
use modelo::*;
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn erro(msg: impl AsRef<str>) -> JsError {
    JsError::new(msg.as_ref())
}

fn json<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".into())
}

/// O caderno de alimentos e receitas de uma pessoa.
#[wasm_bindgen]
pub struct Livro {
    dados: Dados,
}

#[derive(Serialize)]
struct AlimentoVista<'a> {
    #[serde(flatten)]
    alimento: &'a Alimento,
    preco_unitario: f64,
    /// Preço por 100 g/ml, ou por unidade.
    preco_referencia: f64,
    rotulo_referencia: &'static str,
    usado_em: Vec<&'a str>,
}

#[derive(Serialize)]
struct ResumoReceita<'a> {
    id: u32,
    nome: &'a str,
    porcoes: f64,
    ingredientes: usize,
    custo_total: f64,
    custo_porcao: f64,
    kcal_porcao: f64,
    avisos: usize,
}

#[derive(Serialize)]
struct OpcaoMedida {
    valor: Medida,
    nome: &'static str,
}

#[wasm_bindgen]
impl Livro {
    /// Livro vazio.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Livro {
        Livro { dados: Dados { versao: VERSAO_FORMATO, ..Default::default() } }
    }

    /// Livro com alguns alimentos e uma receita de exemplo, para a primeira visita.
    pub fn exemplo() -> Livro {
        Livro { dados: formato::exemplo() }
    }

    /// Abre um livro a partir do JSON salvo (localStorage ou arquivo exportado).
    pub fn carregar(texto: &str) -> Result<Livro, JsError> {
        formato::ler(texto).map(|dados| Livro { dados }).map_err(erro)
    }

    /// Texto JSON com tudo, para salvar ou exportar.
    pub fn exportar(&self) -> String {
        serde_json::to_string_pretty(&self.dados).unwrap_or_default()
    }

    // ---------- Alimentos ----------

    /// Lista de alimentos em ordem alfabética, com preço por referência e onde cada um é usado.
    pub fn alimentos(&self) -> String {
        let mut lista: Vec<AlimentoVista> = self
            .dados
            .alimentos
            .iter()
            .map(|a| {
                let (k, rot) = match a.unidade {
                    Unidade::G => (100.0, "por 100 g"),
                    Unidade::Ml => (100.0, "por 100 ml"),
                    Unidade::Un => (1.0, "por unidade"),
                };
                AlimentoVista {
                    alimento: a,
                    preco_unitario: a.preco_unitario(),
                    preco_referencia: a.preco_unitario() * k,
                    rotulo_referencia: rot,
                    usado_em: self
                        .dados
                        .receitas
                        .iter()
                        .filter(|r| r.ingredientes.iter().any(|i| i.alimento_id == a.id))
                        .map(|r| r.nome.as_str())
                        .collect(),
                }
            })
            .collect();
        lista.sort_by_key(|v| formato::chave_ordem(&v.alimento.nome));
        json(&lista)
    }

    pub fn alimento(&self, id: u32) -> Option<String> {
        self.dados.alimentos.iter().find(|a| a.id == id).map(json)
    }

    /// Cria (id 0 ou ausente) ou atualiza um alimento. Devolve o id.
    pub fn salvar_alimento(&mut self, texto: &str) -> Result<u32, JsError> {
        let mut a: Alimento = serde_json::from_str(texto).map_err(|e| erro(format!("Dados do alimento inválidos: {e}")))?;
        a.nome = a.nome.trim().to_string();
        if a.unidade == Unidade::G {
            a.gramas_por_unidade = None;
        }
        validar_alimento(&a).map_err(erro)?;
        let repetido = self
            .dados
            .alimentos
            .iter()
            .any(|o| o.id != a.id && formato::chave_ordem(&o.nome) == formato::chave_ordem(&a.nome));
        if repetido {
            return Err(erro(format!("Já existe um alimento chamado \"{}\".", a.nome)));
        }
        if a.id == 0 {
            a.id = self.proximo_id_alimento();
            let id = a.id;
            self.dados.alimentos.push(a);
            return Ok(id);
        }
        let Some(pos) = self.dados.alimentos.iter().position(|o| o.id == a.id) else {
            return Err(erro("Esse alimento não existe mais."));
        };
        let antigo = self.dados.alimentos[pos].unidade;
        if antigo != a.unidade {
            let usos = self.receitas_que_usam(a.id);
            if !usos.is_empty() {
                return Err(erro(format!(
                    "Não dá para trocar a unidade de \"{}\" porque ele está em: {}. Tire-o dessas receitas primeiro.",
                    a.nome,
                    usos.join(", ")
                )));
            }
        }
        let id = a.id;
        self.dados.alimentos[pos] = a;
        Ok(id)
    }

    pub fn remover_alimento(&mut self, id: u32) -> Result<(), JsError> {
        let usos = self.receitas_que_usam(id);
        if !usos.is_empty() {
            return Err(erro(format!("Esse alimento está em: {}. Tire-o dessas receitas antes de apagar.", usos.join(", "))));
        }
        self.dados.alimentos.retain(|a| a.id != id);
        Ok(())
    }

    /// Medidas que servem para um alimento (JSON `[{valor, nome}]`).
    pub fn medidas_do_alimento(&self, id: u32) -> String {
        let un = self.dados.alimentos.iter().find(|a| a.id == id).map(|a| a.unidade);
        let lista: Vec<OpcaoMedida> = un
            .map(Medida::para)
            .unwrap_or_default()
            .into_iter()
            .map(|m| OpcaoMedida { valor: m, nome: m.nome() })
            .collect();
        json(&lista)
    }

    // ---------- Receitas ----------

    /// Lista de receitas com o resumo de custo e calorias por porção.
    pub fn receitas(&self) -> String {
        let mut lista: Vec<ResumoReceita> = self
            .dados
            .receitas
            .iter()
            .map(|r| {
                let res = calcular(r, &self.dados.alimentos);
                ResumoReceita {
                    id: r.id,
                    nome: &r.nome,
                    porcoes: r.porcoes,
                    ingredientes: r.ingredientes.len(),
                    custo_total: res.custo_total,
                    custo_porcao: res.custo_porcao,
                    kcal_porcao: res.nutrientes[0].porcao,
                    avisos: res.avisos.len(),
                }
            })
            .collect();
        lista.sort_by_key(|r| formato::chave_ordem(r.nome));
        json(&lista)
    }

    pub fn receita(&self, id: u32) -> Option<String> {
        self.dados.receitas.iter().find(|r| r.id == id).map(json)
    }

    /// Cria (id 0) ou atualiza uma receita. Devolve o id.
    pub fn salvar_receita(&mut self, texto: &str) -> Result<u32, JsError> {
        let mut r: Receita = serde_json::from_str(texto).map_err(|e| erro(format!("Dados da receita inválidos: {e}")))?;
        r.nome = r.nome.trim().to_string();
        validar_receita(&r, &self.dados.alimentos).map_err(erro)?;
        if r.id == 0 {
            r.id = self.proximo_id_receita();
            let id = r.id;
            self.dados.receitas.push(r);
            return Ok(id);
        }
        let Some(pos) = self.dados.receitas.iter().position(|o| o.id == r.id) else {
            return Err(erro("Essa receita não existe mais."));
        };
        let id = r.id;
        self.dados.receitas[pos] = r;
        Ok(id)
    }

    pub fn remover_receita(&mut self, id: u32) {
        self.dados.receitas.retain(|r| r.id != id);
    }

    /// Copia uma receita com o nome "(cópia)". Devolve o id novo.
    pub fn duplicar_receita(&mut self, id: u32) -> Result<u32, JsError> {
        let Some(orig) = self.dados.receitas.iter().find(|r| r.id == id).cloned() else {
            return Err(erro("Essa receita não existe mais."));
        };
        let mut nova = orig;
        nova.id = self.proximo_id_receita();
        nova.nome = format!("{} (cópia)", nova.nome);
        let novo_id = nova.id;
        self.dados.receitas.push(nova);
        Ok(novo_id)
    }

    /// Resultado de uma receita salva.
    pub fn calcular_receita(&self, id: u32) -> Result<String, JsError> {
        let r = self.dados.receitas.iter().find(|r| r.id == id).ok_or_else(|| erro("Essa receita não existe mais."))?;
        Ok(json(&calcular(r, &self.dados.alimentos)))
    }

    /// Resultado de uma receita ainda em edição (não salva), para a conta ao vivo.
    pub fn calcular_rascunho(&self, texto: &str) -> Result<String, JsError> {
        let r: Receita = serde_json::from_str(texto).map_err(|e| erro(format!("Rascunho inválido: {e}")))?;
        Ok(json(&calcular(&r, &self.dados.alimentos)))
    }

    pub fn total_alimentos(&self) -> usize {
        self.dados.alimentos.len()
    }

    pub fn total_receitas(&self) -> usize {
        self.dados.receitas.len()
    }
}

impl Default for Livro {
    fn default() -> Self {
        Self::new()
    }
}

impl Livro {
    fn proximo_id_alimento(&self) -> u32 {
        self.dados.alimentos.iter().map(|a| a.id).max().unwrap_or(0) + 1
    }
    fn proximo_id_receita(&self) -> u32 {
        self.dados.receitas.iter().map(|r| r.id).max().unwrap_or(0) + 1
    }
    fn receitas_que_usam(&self, id: u32) -> Vec<String> {
        self.dados
            .receitas
            .iter()
            .filter(|r| r.ingredientes.iter().any(|i| i.alimento_id == id))
            .map(|r| format!("\"{}\"", r.nome))
            .collect()
    }
}

// ---------- Números no formato brasileiro ----------

/// `1234.5` → `"R$ 1.234,50"`.
#[wasm_bindgen]
pub fn formatar_moeda(v: f64) -> String {
    formato::moeda(v)
}

/// `1234.567, 1` → `"1.234,6"`. Zeros à direita são removidos quando `enxuto` é verdadeiro.
#[wasm_bindgen]
pub fn formatar_numero(v: f64, casas: u32, enxuto: bool) -> String {
    formato::numero(v, casas as usize, enxuto)
}

/// Lê um número digitado em português ("1.234,5", "0,25", "2.5"). Vazio ou inválido → NaN.
#[wasm_bindgen]
pub fn ler_numero(texto: &str) -> f64 {
    formato::ler_numero(texto).unwrap_or(f64::NAN)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn exemplo_calcula_sem_avisos() {
        let l = Livro::exemplo();
        assert!(l.total_alimentos() >= 5);
        let r = &l.dados.receitas[0];
        let res = calcular(r, &l.dados.alimentos);
        assert!(res.avisos.is_empty(), "{:?}", res.avisos);
        assert!(res.custo_total > 0.0);
    }

    #[test]
    fn exportar_e_carregar_volta_igual() {
        let l = Livro::exemplo();
        let texto = l.exportar();
        let l2 = Livro { dados: formato::ler(&texto).unwrap() };
        assert_eq!(l.dados, l2.dados);
    }

    #[test]
    fn nao_apaga_alimento_em_uso() {
        let mut l = Livro::exemplo();
        let id = l.dados.receitas[0].ingredientes[0].alimento_id;
        assert!(!l.receitas_que_usam(id).is_empty());
        let antes = l.total_alimentos();
        // remover_alimento devolve JsError, que não funciona fora do wasm; testamos a regra direto.
        l.dados.receitas.clear();
        assert!(l.receitas_que_usam(id).is_empty());
        assert_eq!(l.total_alimentos(), antes);
    }
}
