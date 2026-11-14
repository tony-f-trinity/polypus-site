//! Polypus · À vista ou parcelado — núcleo em Rust, compilado para WebAssembly.
//!
//! Toda a regra mora aqui: os campos (preço à vista, CDI, opções de parcelamento), a leitura
//! e a formatação em reais, a simulação da aplicação com IR e os textos que a tela mostra.
//! O JavaScript da página só desenha a tela e guarda o JSON no localStorage.

pub mod calculo;
pub mod formato;

use calculo::{juros_embutido, necessario_hoje, simular, taxa_mensal, Simulacao};
use formato::{campo, centavos, moeda, porcentagem2};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub const VERSAO_FORMATO: u32 = 1;
/// Mais que isso não faz sentido (30 anos).
pub const VEZES_MAX: u32 = 360;

fn json<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".into())
}

fn reais(c: i64) -> f64 {
    c as f64 / 100.0
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Opcao {
    #[serde(default)]
    pub id: u32,
    #[serde(default)]
    pub vezes: u32,
    /// Centavos.
    #[serde(default)]
    pub parcela: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Dados {
    #[serde(default)]
    pub versao: u32,
    /// Menor preço à vista, em centavos.
    #[serde(default)]
    pub vista: i64,
    /// CDI ao ano em centésimos de ponto percentual (1490 = 14,90%).
    #[serde(default)]
    pub cdi: i64,
    /// Percentual do CDI em centésimos (10000 = 100%).
    #[serde(default)]
    pub pct_cdi: i64,
    #[serde(default)]
    pub primeira_no_ato: bool,
    #[serde(default)]
    pub opcoes: Vec<Opcao>,
    /// Opção aberta no mês a mês (0 = a melhor).
    #[serde(default)]
    pub selecionada: u32,
}

impl Dados {
    fn arrumar(mut self) -> Dados {
        self.versao = VERSAO_FORMATO;
        self.vista = self.vista.clamp(0, formato::LIMITE);
        self.cdi = self.cdi.clamp(0, 100_000);
        self.pct_cdi = self.pct_cdi.clamp(0, 100_000);
        // os ids são refeitos (1, 2, 3…); a opção aberta acompanha a posição
        let aberta = self.opcoes.iter().position(|o| o.id == self.selecionada && o.id != 0);
        self.selecionada = aberta.map(|k| k as u32 + 1).unwrap_or(0);
        for (k, o) in self.opcoes.iter_mut().enumerate() {
            o.id = k as u32 + 1;
            o.vezes = o.vezes.min(VEZES_MAX);
            o.parcela = o.parcela.clamp(0, formato::LIMITE);
        }
        if !self.opcoes.iter().any(|o| o.id == self.selecionada) {
            self.selecionada = 0;
        }
        self
    }

    fn proximo_id(&self) -> u32 {
        self.opcoes.iter().map(|o| o.id).max().unwrap_or(0) + 1
    }

    fn opcao(&mut self, id: u32) -> Option<&mut Opcao> {
        self.opcoes.iter_mut().find(|o| o.id == id)
    }
}

fn exemplo_dados() -> Dados {
    let o = |vezes: u32, parcela: i64| Opcao { id: 0, vezes, parcela };
    Dados {
        versao: VERSAO_FORMATO,
        vista: 95000,
        cdi: 1490,
        pct_cdi: 10000,
        primeira_no_ato: false,
        opcoes: vec![o(3, 33000), o(6, 16667), o(10, 10000), o(12, 8200)],
        selecionada: 0,
    }
    .arrumar()
}

/// Lê "12", "12x", "12 vezes" → 12. Vazio ou inválido → 0.
fn ler_vezes(texto: &str) -> u32 {
    let digitos: String = texto.chars().take_while(|c| !c.is_ascii_alphabetic()).filter(|c| c.is_ascii_digit()).collect();
    digitos.parse::<u64>().map(|v| v.min(VEZES_MAX as u64) as u32).unwrap_or(0)
}

// ---------- O que a tela recebe ----------

#[derive(Serialize)]
struct OpcaoCampo {
    id: u32,
    vezes: String,
    parcela: String,
}

#[derive(Serialize)]
struct Veredito {
    /// "parcelar", "vista" ou "empate".
    tipo: &'static str,
    selo: &'static str,
    titulo: String,
    texto: String,
}

#[derive(Serialize)]
struct Linha {
    id: u32,
    plano: String,
    total: String,
    juros: String,
    aplicar: String,
    diferenca: String,
    /// "parcelar", "vista" ou "empate".
    tipo: &'static str,
    selo: &'static str,
    /// -1 a 1, relativo à maior diferença em valor absoluto.
    barra: f64,
    sobra: String,
    sobra_falta: bool,
    selecionada: bool,
}

#[derive(Serialize)]
struct MesVista {
    mes: String,
    saldo_antes: String,
    rendimento: String,
    resgate: String,
    ir: String,
    parcela: String,
    saldo_depois: String,
    faltou: bool,
}

#[derive(Serialize)]
struct Detalhe {
    titulo: String,
    sub: String,
    meses: Vec<MesVista>,
    total_rendimento: String,
    total_ir: String,
    total_pago: String,
    fim: String,
    fim_ok: bool,
}

#[derive(Serialize)]
struct TotalOpcao {
    id: u32,
    texto: String,
}

#[derive(Serialize)]
struct Resultado {
    /// Mensagem quando ainda falta informação; o resto vem vazio.
    falta: Option<String>,
    totais: Vec<TotalOpcao>,
    veredito: Option<Veredito>,
    linhas: Vec<Linha>,
    detalhe: Option<Detalhe>,
}

struct Conta {
    id: u32,
    vezes: u32,
    parcela: f64,
    necessario: f64,
    /// À vista − necessário, em centavos (positivo: parcelar deixa dinheiro no bolso).
    diferenca: i64,
    juros: Option<f64>,
    sim: Simulacao,
}

fn tipo_e_selo(diferenca: i64) -> (&'static str, &'static str) {
    match diferenca {
        d if d > 0 => ("parcelar", "Parcelar"),
        d if d < 0 => ("vista", "À vista"),
        _ => ("empate", "Empate"),
    }
}

fn plano(vezes: u32, parcela: f64) -> String {
    format!("{vezes}x {}", moeda(centavos(parcela)))
}

/// A tela: o que o preço à vista e as opções de parcelamento rendem lado a lado.
#[wasm_bindgen]
pub struct Comparador {
    dados: Dados,
}

#[wasm_bindgen]
impl Comparador {
    /// Campos vazios.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Comparador {
        Comparador { dados: Dados { versao: VERSAO_FORMATO, pct_cdi: 10000, ..Default::default() } }
    }

    /// Valores de exemplo, para a primeira visita.
    pub fn exemplo() -> Comparador {
        Comparador { dados: exemplo_dados() }
    }

    /// Abre o que foi salvo no localStorage.
    pub fn carregar(texto: &str) -> Result<Comparador, JsError> {
        let dados: Dados = serde_json::from_str(texto).map_err(|e| JsError::new(&format!("Dados inválidos: {e}")))?;
        Ok(Comparador { dados: dados.arrumar() })
    }

    pub fn exportar(&self) -> String {
        serde_json::to_string(&self.dados).unwrap_or_default()
    }

    pub fn restaurar_exemplo(&mut self) {
        self.dados = exemplo_dados();
    }

    // ---------- Campos gerais ----------

    pub fn vista_campo(&self) -> String {
        if self.dados.vista > 0 { campo(self.dados.vista) } else { String::new() }
    }
    pub fn definir_vista(&mut self, texto: &str) {
        self.dados.vista = formato::ler_centavos(texto);
    }

    pub fn cdi_campo(&self) -> String {
        campo(self.dados.cdi)
    }
    pub fn definir_cdi(&mut self, texto: &str) {
        self.dados.cdi = formato::ler_centavos(texto).min(100_000);
    }

    pub fn pct_campo(&self) -> String {
        let s = campo(self.dados.pct_cdi);
        s.strip_suffix(",00").map(str::to_string).unwrap_or(s)
    }
    pub fn definir_pct(&mut self, texto: &str) {
        self.dados.pct_cdi = formato::ler_centavos(texto).min(100_000);
    }

    pub fn primeira_no_ato(&self) -> bool {
        self.dados.primeira_no_ato
    }
    pub fn definir_primeira_no_ato(&mut self, no_ato: bool) {
        self.dados.primeira_no_ato = no_ato;
    }

    // ---------- Opções de parcelamento ----------

    /// JSON `[{id, vezes, parcela}]` com os textos dos campos.
    pub fn opcoes(&self) -> String {
        json(
            &self
                .dados
                .opcoes
                .iter()
                .map(|o| OpcaoCampo {
                    id: o.id,
                    vezes: if o.vezes > 0 { o.vezes.to_string() } else { String::new() },
                    parcela: if o.parcela > 0 { campo(o.parcela) } else { String::new() },
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn total_opcoes(&self) -> usize {
        self.dados.opcoes.len()
    }

    /// Nova opção com uma vez a mais que a última. Devolve o id.
    pub fn adicionar(&mut self) -> u32 {
        let id = self.dados.proximo_id();
        let vezes = self.dados.opcoes.last().map(|o| (o.vezes + 1).min(VEZES_MAX)).unwrap_or(2);
        self.dados.opcoes.push(Opcao { id, vezes, parcela: 0 });
        id
    }

    pub fn remover(&mut self, id: u32) {
        self.dados.opcoes.retain(|o| o.id != id);
        if self.dados.selecionada == id {
            self.dados.selecionada = 0;
        }
    }

    pub fn definir_vezes(&mut self, id: u32, texto: &str) {
        if let Some(o) = self.dados.opcao(id) {
            o.vezes = ler_vezes(texto);
        }
    }

    pub fn definir_parcela(&mut self, id: u32, texto: &str) {
        if let Some(o) = self.dados.opcao(id) {
            o.parcela = formato::ler_centavos(texto);
        }
    }

    pub fn parcela_campo(&self, id: u32) -> String {
        self.dados.opcoes.iter().find(|o| o.id == id).filter(|o| o.parcela > 0).map(|o| campo(o.parcela)).unwrap_or_default()
    }

    /// Abre esta opção no mês a mês.
    pub fn selecionar(&mut self, id: u32) {
        self.dados.selecionada = id;
    }

    // ---------- Resultado ----------

    /// Tudo o que a tela mostra, em JSON, com os textos prontos.
    pub fn resultado(&self) -> String {
        json(&self.calcular())
    }
}

impl Default for Comparador {
    fn default() -> Self {
        Comparador::new()
    }
}

impl Comparador {
    fn calcular(&self) -> Resultado {
        let d = &self.dados;
        let validas: Vec<&Opcao> = d.opcoes.iter().filter(|o| o.vezes > 0 && o.parcela > 0).collect();
        let totais = validas
            .iter()
            .map(|o| TotalOpcao { id: o.id, texto: format!("total {}", moeda(o.parcela * o.vezes as i64)) })
            .collect();
        let vazio = |falta: &str, totais| Resultado { falta: Some(falta.into()), totais, veredito: None, linhas: vec![], detalhe: None };
        if d.vista <= 0 {
            return vazio("Informe o menor valor à vista.", totais);
        }
        if validas.is_empty() {
            return vazio("Adicione pelo menos uma opção de parcelamento com número de vezes e valor da parcela.", totais);
        }

        let vista = reais(d.vista);
        let cdi = d.cdi as f64 / 10000.0;
        let pct = d.pct_cdi as f64 / 10000.0;
        let taxa = taxa_mensal(cdi, pct);
        let no_ato = d.primeira_no_ato;

        let contas: Vec<Conta> = validas
            .iter()
            .map(|o| {
                let parcela = reais(o.parcela);
                let necessario = necessario_hoje(o.vezes, parcela, no_ato, taxa);
                Conta {
                    id: o.id,
                    vezes: o.vezes,
                    parcela,
                    necessario,
                    diferenca: d.vista - centavos(necessario),
                    juros: juros_embutido(vista, o.vezes, parcela, no_ato),
                    sim: simular(vista, o.vezes, parcela, no_ato, taxa),
                }
            })
            .collect();

        // a melhor: a que deixa mais dinheiro hoje (na dúvida, a primeira)
        let melhor = contas.iter().fold(&contas[0], |a, b| if b.diferenca > a.diferenca { b } else { a });
        let maior = contas.iter().map(|c| c.diferenca.abs()).max().unwrap_or(0).max(1) as f64;
        let aberta = contas.iter().find(|c| c.id == d.selecionada).unwrap_or(melhor);

        let pct_txt = self.pct_campo();
        let veredito = if melhor.diferenca > 0 {
            Veredito {
                tipo: "parcelar",
                selo: "Parcelar compensa",
                titulo: format!("Em {}x de {}, você fica com {} a mais hoje", melhor.vezes, moeda(centavos(melhor.parcela)), moeda(melhor.diferenca)),
                texto: format!(
                    "Basta aplicar {} a {pct_txt}% do CDI para pagar todas as parcelas, em vez de gastar {} à vista. Deixando os {} aplicados, sobram {} líquidos depois da última parcela.",
                    moeda(centavos(melhor.necessario)),
                    moeda(d.vista),
                    moeda(d.vista),
                    moeda(centavos(melhor.sim.sobra_liquida))
                ),
            }
        } else if melhor.diferenca == 0 {
            Veredito {
                tipo: "empate",
                selo: "Empate",
                titulo: format!("Em {}x de {}, parcelar dá o mesmo que pagar à vista", melhor.vezes, moeda(centavos(melhor.parcela))),
                texto: format!(
                    "O que a aplicação rende depois do IR cobre exatamente o juros embutido. Aplicando {} você paga todas as parcelas e não sobra nada.",
                    moeda(d.vista)
                ),
            }
        } else {
            Veredito {
                tipo: "vista",
                selo: "Pague à vista",
                titulo: format!("Pagar à vista economiza {} em relação à melhor opção parcelada", moeda(-melhor.diferenca)),
                texto: format!(
                    "Mesmo a melhor opção ({}x de {}) exige aplicar {} hoje para cobrir as parcelas, mais que os {} do preço à vista. O juros embutido no parcelamento supera o que a aplicação rende depois do IR.",
                    melhor.vezes,
                    moeda(centavos(melhor.parcela)),
                    moeda(centavos(melhor.necessario)),
                    moeda(d.vista)
                ),
            }
        };

        let linhas = contas
            .iter()
            .map(|c| {
                let (tipo, selo) = tipo_e_selo(c.diferenca);
                Linha {
                    id: c.id,
                    plano: plano(c.vezes, c.parcela),
                    total: moeda(centavos(c.parcela * c.vezes as f64)),
                    juros: c.juros.map(|j| format!("{} a.m.", porcentagem2(j))).unwrap_or_else(|| "—".into()),
                    aplicar: moeda(centavos(c.necessario)),
                    diferenca: if c.diferenca > 0 { format!("+{}", moeda(c.diferenca)) } else { moeda(c.diferenca) },
                    tipo,
                    selo,
                    barra: c.diferenca as f64 / maior,
                    sobra: if c.sim.cobre() { moeda(centavos(c.sim.sobra_liquida)) } else { format!("falta {}", moeda(centavos(c.sim.falta))) },
                    sobra_falta: !c.sim.cobre(),
                    selecionada: c.id == aberta.id,
                }
            })
            .collect();

        let s = &aberta.sim;
        let ano = (1.0 + taxa).powi(12) - 1.0;
        let detalhe = Detalhe {
            titulo: format!("Mês a mês: {}x de {}", aberta.vezes, moeda(centavos(aberta.parcela))),
            sub: format!("Aplicando {} hoje · rende {} ao mês bruto ({} ao ano)", moeda(d.vista), porcentagem2(taxa), porcentagem2(ano)),
            meses: s
                .meses
                .iter()
                .map(|m| MesVista {
                    mes: if m.mes == 0 { "No ato".into() } else { format!("{}º ({} dias)", m.mes, m.dias) },
                    saldo_antes: moeda(centavos(m.saldo_antes)),
                    rendimento: moeda(centavos(m.rendimento)),
                    resgate: moeda(centavos(m.resgate)),
                    ir: moeda(centavos(m.ir)),
                    parcela: if m.faltou { format!("só {}", moeda(centavos(m.pago))) } else { moeda(centavos(m.pago)) },
                    saldo_depois: moeda(centavos(m.saldo_depois)),
                    faltou: m.faltou,
                })
                .collect(),
            total_rendimento: moeda(centavos(s.meses.iter().map(|m| m.rendimento).sum())),
            total_ir: moeda(centavos(s.meses.iter().map(|m| m.ir).sum())),
            total_pago: moeda(centavos(s.meses.iter().map(|m| m.pago).sum())),
            fim: if s.cobre() { format!("sobra {}", moeda(centavos(s.sobra_liquida))) } else { format!("falta {}", moeda(centavos(s.falta))) },
            fim_ok: s.cobre(),
        };

        Resultado { falta: None, totais, veredito: Some(veredito), linhas, detalhe: Some(detalhe) }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn resultado(c: &Comparador) -> serde_json::Value {
        serde_json::from_str(&c.resultado()).unwrap()
    }

    #[test]
    fn exemplo_mostra_as_quatro_opcoes() {
        let c = Comparador::exemplo();
        let r = resultado(&c);
        assert!(r["falta"].is_null());
        assert_eq!(r["linhas"].as_array().unwrap().len(), 4);
        // 12x de R$ 82,00 sobre R$ 950,00: juros embutido abaixo do que a aplicação rende
        assert_eq!(r["veredito"]["tipo"], "parcelar");
        assert_eq!(r["linhas"][2]["aplicar"], "R$\u{202f}950,88");
        assert_eq!(r["linhas"][2]["tipo"], "vista");
    }

    #[test]
    fn pede_o_que_falta() {
        let mut c = Comparador::new();
        assert!(resultado(&c)["falta"].as_str().unwrap().contains("à vista"));
        c.definir_vista("950");
        assert!(resultado(&c)["falta"].as_str().unwrap().contains("opção"));
        let id = c.adicionar();
        c.definir_parcela(id, "500");
        assert!(resultado(&c)["falta"].is_null());
    }

    #[test]
    fn le_os_campos() {
        let mut c = Comparador::new();
        c.definir_cdi("14,9");
        assert_eq!(c.cdi_campo(), "14,90");
        c.definir_pct("110");
        assert_eq!(c.pct_campo(), "110");
        c.definir_pct("102,5");
        assert_eq!(c.pct_campo(), "102,50");
        assert_eq!(ler_vezes("12x"), 12);
        assert_eq!(ler_vezes(" 6 vezes"), 6);
        assert_eq!(ler_vezes("abc"), 0);
        assert_eq!(ler_vezes("9999"), VEZES_MAX);
    }

    #[test]
    fn salva_e_abre() {
        let mut c = Comparador::exemplo();
        c.definir_primeira_no_ato(true);
        c.remover(1);
        let d = Comparador::carregar(&c.exportar()).unwrap();
        assert!(d.primeira_no_ato());
        assert_eq!(d.total_opcoes(), 3);
        // JsError só existe dentro do WebAssembly; aqui basta ver que o JSON é recusado
        assert!(serde_json::from_str::<Dados>("{lixo").is_err());
    }
}
