//! Tipos de dados: alimentos, receitas e o "livro" que guarda tudo.

use serde::{Deserialize, Serialize};

/// Versão do formato salvo. Aumente quando mudar a estrutura de forma incompatível.
pub const VERSAO_FORMATO: u32 = 1;

/// Unidade em que o alimento é comprado e em que a tabela nutricional é dada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unidade {
    /// Gramas (sólidos: farinha, carne, queijo...)
    G,
    /// Mililitros (líquidos: leite, óleo, água...)
    Ml,
    /// Unidades (ovo, pão, lata...)
    Un,
}

impl Unidade {
    pub fn sigla(self) -> &'static str {
        match self {
            Unidade::G => "g",
            Unidade::Ml => "ml",
            Unidade::Un => "un",
        }
    }
}

/// Medida usada num ingrediente da receita. Cada uma é convertida para a unidade-base do alimento.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Medida {
    G,
    Kg,
    Ml,
    L,
    Xicara,
    ColherSopa,
    ColherCha,
    Un,
}

impl Medida {
    /// Unidade-base com que a medida é compatível e o fator de conversão para ela.
    pub fn base(self) -> (Unidade, f64) {
        match self {
            Medida::G => (Unidade::G, 1.0),
            Medida::Kg => (Unidade::G, 1000.0),
            Medida::Ml => (Unidade::Ml, 1.0),
            Medida::L => (Unidade::Ml, 1000.0),
            Medida::Xicara => (Unidade::Ml, 240.0),
            Medida::ColherSopa => (Unidade::Ml, 15.0),
            Medida::ColherCha => (Unidade::Ml, 5.0),
            Medida::Un => (Unidade::Un, 1.0),
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Medida::G => "g",
            Medida::Kg => "kg",
            Medida::Ml => "ml",
            Medida::L => "L",
            Medida::Xicara => "xícara (240 ml)",
            Medida::ColherSopa => "colher de sopa (15 ml)",
            Medida::ColherCha => "colher de chá (5 ml)",
            Medida::Un => "unidade",
        }
    }

    pub const TODAS: [Medida; 8] = [
        Medida::G,
        Medida::Kg,
        Medida::Ml,
        Medida::L,
        Medida::Xicara,
        Medida::ColherSopa,
        Medida::ColherCha,
        Medida::Un,
    ];

    /// Medidas que fazem sentido para um alimento com esta unidade-base.
    pub fn para(unidade: Unidade) -> Vec<Medida> {
        Medida::TODAS.iter().copied().filter(|m| m.base().0 == unidade).collect()
    }
}

/// Os nutrientes da tabela nutricional brasileira (RDC 429/2020), mais açúcares totais.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Nutrientes {
    pub energia_kcal: f64,
    pub carboidratos_g: f64,
    pub acucares_g: f64,
    pub proteinas_g: f64,
    pub gorduras_totais_g: f64,
    pub gorduras_saturadas_g: f64,
    pub fibras_g: f64,
    pub sodio_mg: f64,
}

impl Nutrientes {
    pub fn escalar(&self, k: f64) -> Nutrientes {
        Nutrientes {
            energia_kcal: self.energia_kcal * k,
            carboidratos_g: self.carboidratos_g * k,
            acucares_g: self.acucares_g * k,
            proteinas_g: self.proteinas_g * k,
            gorduras_totais_g: self.gorduras_totais_g * k,
            gorduras_saturadas_g: self.gorduras_saturadas_g * k,
            fibras_g: self.fibras_g * k,
            sodio_mg: self.sodio_mg * k,
        }
    }

    pub fn somar(&mut self, o: &Nutrientes) {
        self.energia_kcal += o.energia_kcal;
        self.carboidratos_g += o.carboidratos_g;
        self.acucares_g += o.acucares_g;
        self.proteinas_g += o.proteinas_g;
        self.gorduras_totais_g += o.gorduras_totais_g;
        self.gorduras_saturadas_g += o.gorduras_saturadas_g;
        self.fibras_g += o.fibras_g;
        self.sodio_mg += o.sodio_mg;
    }

    pub fn valores(&self) -> [(&'static str, f64); 8] {
        [
            ("energia_kcal", self.energia_kcal),
            ("carboidratos_g", self.carboidratos_g),
            ("acucares_g", self.acucares_g),
            ("proteinas_g", self.proteinas_g),
            ("gorduras_totais_g", self.gorduras_totais_g),
            ("gorduras_saturadas_g", self.gorduras_saturadas_g),
            ("fibras_g", self.fibras_g),
            ("sodio_mg", self.sodio_mg),
        ]
    }
}

/// Um alimento cadastrado: quanto custa e o que ele tem.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Alimento {
    #[serde(default)]
    pub id: u32,
    pub nome: String,
    pub unidade: Unidade,
    /// Preço pago pela embalagem, em reais.
    pub preco: f64,
    /// Quanto vem na embalagem, na unidade-base (ex.: 1000 g, 900 ml, 12 un).
    pub qtd_embalagem: f64,
    /// A quantidade a que a tabela nutricional se refere (normalmente 100 g ou 100 ml; 1 un).
    pub base_nutricional: f64,
    #[serde(default)]
    pub nutrientes: Nutrientes,
    /// Gramas por unidade-base: densidade (g/ml) para líquidos, peso de uma unidade para "un".
    /// Usado só para estimar o peso da receita. Para "g" é sempre 1.
    #[serde(default)]
    pub gramas_por_unidade: Option<f64>,
    #[serde(default)]
    pub obs: String,
}

impl Alimento {
    /// Preço de uma unidade-base (R$/g, R$/ml ou R$/un).
    pub fn preco_unitario(&self) -> f64 {
        if self.qtd_embalagem > 0.0 {
            self.preco / self.qtd_embalagem
        } else {
            0.0
        }
    }

    /// Quantos gramas pesa uma unidade-base, se der para saber.
    pub fn gramas(&self) -> Option<f64> {
        match self.unidade {
            Unidade::G => Some(1.0),
            Unidade::Ml => Some(self.gramas_por_unidade.unwrap_or(1.0)),
            Unidade::Un => self.gramas_por_unidade.filter(|g| *g > 0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ingrediente {
    pub alimento_id: u32,
    pub quantidade: f64,
    pub medida: Medida,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Receita {
    #[serde(default)]
    pub id: u32,
    pub nome: String,
    /// Em quantas porções a receita rende.
    pub porcoes: f64,
    /// Peso final da receita pronta, em gramas (depois de assar/cozinhar). Opcional.
    #[serde(default)]
    pub peso_final_g: Option<f64>,
    /// Margem sobre o custo para sugerir um preço de venda (ex.: 150 = custo × 2,5). Opcional.
    #[serde(default)]
    pub margem_pct: Option<f64>,
    #[serde(default)]
    pub ingredientes: Vec<Ingrediente>,
    #[serde(default)]
    pub preparo: String,
}

/// Tudo o que o usuário cadastrou. É o que vai para o localStorage e para o arquivo exportado.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Dados {
    #[serde(default = "versao_padrao")]
    pub versao: u32,
    #[serde(default)]
    pub alimentos: Vec<Alimento>,
    #[serde(default)]
    pub receitas: Vec<Receita>,
}

fn versao_padrao() -> u32 {
    VERSAO_FORMATO
}
