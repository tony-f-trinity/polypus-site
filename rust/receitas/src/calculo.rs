//! Regras de validação e o cálculo de custo e nutrição de uma receita.

use crate::modelo::*;
use serde::Serialize;

/// Valores diários de referência para adultos (ANVISA, IN 75/2020), na ordem de `Nutrientes::valores`.
/// Açúcares totais não têm VD, por isso ficam com `None`.
pub const VD: [Option<f64>; 8] = [
    Some(2000.0), // energia kcal
    Some(300.0),  // carboidratos g
    None,         // açúcares totais
    Some(50.0),   // proteínas g
    Some(65.0),   // gorduras totais g
    Some(20.0),   // gorduras saturadas g
    Some(25.0),   // fibras g
    Some(2000.0), // sódio mg
];

fn finito_nao_negativo(v: f64) -> bool {
    v.is_finite() && v >= 0.0
}

pub fn validar_alimento(a: &Alimento) -> Result<(), String> {
    if a.nome.trim().is_empty() {
        return Err("Dê um nome ao alimento.".into());
    }
    if !finito_nao_negativo(a.preco) {
        return Err("O preço precisa ser um número igual ou maior que zero.".into());
    }
    if !(a.qtd_embalagem.is_finite() && a.qtd_embalagem > 0.0) {
        return Err(format!(
            "Informe quanto vem na embalagem, em {} (maior que zero).",
            a.unidade.sigla()
        ));
    }
    if !(a.base_nutricional.is_finite() && a.base_nutricional > 0.0) {
        return Err("A quantidade de referência da tabela nutricional precisa ser maior que zero.".into());
    }
    for (nome, v) in a.nutrientes.valores() {
        if !finito_nao_negativo(v) {
            return Err(format!("O valor de \"{}\" não pode ser negativo.", rotulo_nutriente(nome)));
        }
    }
    if let Some(g) = a.gramas_por_unidade {
        if !(g.is_finite() && g > 0.0) {
            return Err("O peso por unidade precisa ser maior que zero (ou fique em branco).".into());
        }
    }
    Ok(())
}

pub fn validar_receita(r: &Receita, alimentos: &[Alimento]) -> Result<(), String> {
    if r.nome.trim().is_empty() {
        return Err("Dê um nome à receita.".into());
    }
    if !(r.porcoes.is_finite() && r.porcoes > 0.0) {
        return Err("O rendimento precisa ser de pelo menos uma porção (pode ser fracionado, ex.: 0,5).".into());
    }
    if let Some(p) = r.peso_final_g {
        if !(p.is_finite() && p > 0.0) {
            return Err("O peso final precisa ser maior que zero (ou fique em branco).".into());
        }
    }
    if let Some(m) = r.margem_pct {
        if !finito_nao_negativo(m) {
            return Err("A margem não pode ser negativa.".into());
        }
    }
    for (i, ing) in r.ingredientes.iter().enumerate() {
        let n = i + 1;
        let Some(a) = alimentos.iter().find(|a| a.id == ing.alimento_id) else {
            return Err(format!("Escolha o alimento do ingrediente {n}."));
        };
        if !(ing.quantidade.is_finite() && ing.quantidade > 0.0) {
            return Err(format!("A quantidade de \"{}\" precisa ser maior que zero.", a.nome));
        }
        if ing.medida.base().0 != a.unidade {
            return Err(format!(
                "\"{}\" é medido em {}; a medida \"{}\" não serve para ele.",
                a.nome,
                a.unidade.sigla(),
                ing.medida.nome()
            ));
        }
    }
    Ok(())
}

pub fn rotulo_nutriente(chave: &str) -> &'static str {
    match chave {
        "energia_kcal" => "Valor energético",
        "carboidratos_g" => "Carboidratos",
        "acucares_g" => "Açúcares totais",
        "proteinas_g" => "Proteínas",
        "gorduras_totais_g" => "Gorduras totais",
        "gorduras_saturadas_g" => "Gorduras saturadas",
        "fibras_g" => "Fibra alimentar",
        "sodio_mg" => "Sódio",
        _ => "",
    }
}

#[derive(Debug, Serialize)]
pub struct LinhaIngrediente {
    /// Posição do ingrediente na receita (começa em 0).
    pub indice: usize,
    pub alimento_id: u32,
    pub nome: String,
    pub quantidade: f64,
    pub medida: Medida,
    /// Quantidade convertida para a unidade-base do alimento.
    pub quantidade_base: f64,
    pub unidade_base: Unidade,
    pub peso_g: Option<f64>,
    pub custo: f64,
    pub parcela_custo_pct: f64,
    pub energia_kcal: f64,
}

#[derive(Debug, Serialize)]
pub struct LinhaNutriente {
    pub chave: &'static str,
    pub nome: &'static str,
    pub unidade: &'static str,
    pub total: f64,
    pub porcao: f64,
    pub por_100g: Option<f64>,
    /// %VD de uma porção.
    pub vd_porcao_pct: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct Resultado {
    pub custo_total: f64,
    pub custo_porcao: f64,
    pub preco_sugerido_total: Option<f64>,
    pub preco_sugerido_porcao: Option<f64>,
    pub porcoes: f64,
    /// Soma dos pesos dos ingredientes crus (só se todos tiverem peso conhecido).
    pub peso_cru_g: Option<f64>,
    /// Peso usado para "por 100 g": o peso final informado ou, na falta dele, o peso cru.
    pub peso_final_g: Option<f64>,
    pub peso_porcao_g: Option<f64>,
    pub nutrientes: Vec<LinhaNutriente>,
    pub ingredientes: Vec<LinhaIngrediente>,
    pub avisos: Vec<String>,
}

/// Calcula custo e nutrição. Ingredientes inválidos não derrubam o cálculo: viram avisos,
/// para a tela poder mostrar um resultado parcial enquanto a pessoa ainda está editando.
pub fn calcular(r: &Receita, alimentos: &[Alimento]) -> Resultado {
    let mut avisos = Vec::new();
    let mut linhas = Vec::new();
    let mut total = Nutrientes::default();
    let mut custo_total = 0.0;
    let mut peso_cru = Some(0.0);

    for (i, ing) in r.ingredientes.iter().enumerate() {
        let Some(a) = alimentos.iter().find(|a| a.id == ing.alimento_id) else {
            avisos.push(format!("O ingrediente {} não tem alimento escolhido e ficou de fora.", i + 1));
            continue;
        };
        let (base, fator) = ing.medida.base();
        if base != a.unidade {
            avisos.push(format!(
                "\"{}\" é medido em {}; troque a medida \"{}\". Ele ficou de fora da conta.",
                a.nome,
                a.unidade.sigla(),
                ing.medida.nome()
            ));
            continue;
        }
        if !(ing.quantidade.is_finite() && ing.quantidade > 0.0) {
            avisos.push(format!("\"{}\" está sem quantidade e ficou de fora.", a.nome));
            continue;
        }
        let qtd = ing.quantidade * fator;
        let custo = qtd * a.preco_unitario();
        let nut = a.nutrientes.escalar(qtd / a.base_nutricional);
        let peso = a.gramas().map(|g| g * qtd);
        if a.preco == 0.0 {
            avisos.push(format!("\"{}\" está com preço zero.", a.nome));
        }
        match (peso, peso_cru) {
            (Some(p), Some(t)) => peso_cru = Some(t + p),
            (None, _) => {
                if peso_cru.is_some() {
                    avisos.push(format!(
                        "Sem o peso de uma unidade de \"{}\", não dá para somar o peso da receita. Informe o peso final ou o peso por unidade do alimento.",
                        a.nome
                    ));
                }
                peso_cru = None;
            }
            _ => {}
        }
        custo_total += custo;
        total.somar(&nut);
        linhas.push(LinhaIngrediente {
            indice: i,
            alimento_id: a.id,
            nome: a.nome.clone(),
            quantidade: ing.quantidade,
            medida: ing.medida,
            quantidade_base: qtd,
            unidade_base: a.unidade,
            peso_g: peso,
            custo,
            parcela_custo_pct: 0.0,
            energia_kcal: nut.energia_kcal,
        });
    }
    for l in &mut linhas {
        l.parcela_custo_pct = if custo_total > 0.0 { l.custo / custo_total * 100.0 } else { 0.0 };
    }

    let porcoes = if r.porcoes.is_finite() && r.porcoes > 0.0 { r.porcoes } else { 1.0 };
    if linhas.is_empty() {
        peso_cru = None;
    }
    let peso_final = r.peso_final_g.filter(|p| *p > 0.0).or(peso_cru);
    let porcao = total.escalar(1.0 / porcoes);
    let por_100 = peso_final.map(|p| total.escalar(100.0 / p));
    let margem = r.margem_pct.filter(|m| m.is_finite() && *m >= 0.0);

    let nutrientes = total
        .valores()
        .iter()
        .zip(porcao.valores().iter())
        .enumerate()
        .map(|(i, ((chave, t), (_, p)))| LinhaNutriente {
            chave,
            nome: rotulo_nutriente(chave),
            unidade: match *chave {
                "energia_kcal" => "kcal",
                "sodio_mg" => "mg",
                _ => "g",
            },
            total: *t,
            porcao: *p,
            por_100g: por_100.map(|n| n.valores()[i].1),
            vd_porcao_pct: VD[i].map(|vd| p / vd * 100.0),
        })
        .collect();

    Resultado {
        custo_total,
        custo_porcao: custo_total / porcoes,
        preco_sugerido_total: margem.map(|m| custo_total * (1.0 + m / 100.0)),
        preco_sugerido_porcao: margem.map(|m| custo_total / porcoes * (1.0 + m / 100.0)),
        porcoes,
        peso_cru_g: peso_cru,
        peso_final_g: peso_final,
        peso_porcao_g: peso_final.map(|p| p / porcoes),
        nutrientes,
        ingredientes: linhas,
        avisos,
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn farinha() -> Alimento {
        Alimento {
            id: 1,
            nome: "Farinha".into(),
            unidade: Unidade::G,
            preco: 6.0,
            qtd_embalagem: 1000.0,
            base_nutricional: 100.0,
            nutrientes: Nutrientes { energia_kcal: 360.0, carboidratos_g: 75.0, proteinas_g: 10.0, sodio_mg: 2.0, ..Default::default() },
            gramas_por_unidade: None,
            obs: String::new(),
        }
    }
    fn leite() -> Alimento {
        Alimento {
            id: 2,
            nome: "Leite".into(),
            unidade: Unidade::Ml,
            preco: 5.0,
            qtd_embalagem: 1000.0,
            base_nutricional: 200.0,
            nutrientes: Nutrientes { energia_kcal: 120.0, proteinas_g: 6.0, sodio_mg: 100.0, ..Default::default() },
            gramas_por_unidade: Some(1.03),
            obs: String::new(),
        }
    }
    fn ovo(peso: Option<f64>) -> Alimento {
        Alimento {
            id: 3,
            nome: "Ovo".into(),
            unidade: Unidade::Un,
            preco: 12.0,
            qtd_embalagem: 12.0,
            base_nutricional: 1.0,
            nutrientes: Nutrientes { energia_kcal: 70.0, proteinas_g: 6.0, ..Default::default() },
            gramas_por_unidade: peso,
            obs: String::new(),
        }
    }
    fn receita(ings: Vec<Ingrediente>) -> Receita {
        Receita { id: 1, nome: "Teste".into(), porcoes: 4.0, peso_final_g: None, margem_pct: Some(100.0), ingredientes: ings, preparo: String::new() }
    }
    fn ing(id: u32, q: f64, m: Medida) -> Ingrediente {
        Ingrediente { alimento_id: id, quantidade: q, medida: m }
    }
    fn perto(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn custo_e_nutricao_com_conversoes() {
        let als = vec![farinha(), leite(), ovo(Some(50.0))];
        let r = receita(vec![ing(1, 0.5, Medida::Kg), ing(2, 1.0, Medida::Xicara), ing(3, 2.0, Medida::Un)]);
        let res = calcular(&r, &als);
        // 500 g × 0,006 + 240 ml × 0,005 + 2 × 1
        assert!(perto(res.custo_total, 3.0 + 1.2 + 2.0));
        assert!(perto(res.custo_porcao, 6.2 / 4.0));
        assert!(perto(res.preco_sugerido_total.unwrap(), 12.4));
        // 1800 + 144 + 140 kcal
        assert!(perto(res.nutrientes[0].total, 2084.0));
        assert!(perto(res.nutrientes[0].porcao, 521.0));
        // 500 + 240 × 1,03 + 100
        assert!(perto(res.peso_cru_g.unwrap(), 847.2));
        assert!(res.avisos.is_empty());
    }

    #[test]
    fn peso_final_manda_no_por_100g() {
        let als = vec![farinha()];
        let mut r = receita(vec![ing(1, 200.0, Medida::G)]);
        r.peso_final_g = Some(360.0);
        let res = calcular(&r, &als);
        // 720 kcal em 360 g = 200 kcal/100 g
        assert!(perto(res.nutrientes[0].por_100g.unwrap(), 200.0));
        assert!(perto(res.peso_porcao_g.unwrap(), 90.0));
    }

    #[test]
    fn unidade_sem_peso_desliga_por_100g() {
        let als = vec![farinha(), ovo(None)];
        let r = receita(vec![ing(1, 100.0, Medida::G), ing(3, 1.0, Medida::Un)]);
        let res = calcular(&r, &als);
        assert!(res.peso_cru_g.is_none());
        assert!(res.nutrientes[0].por_100g.is_none());
        assert_eq!(res.avisos.len(), 1);
    }

    #[test]
    fn medida_errada_vira_aviso_e_erro_na_validacao() {
        let als = vec![farinha()];
        let r = receita(vec![ing(1, 1.0, Medida::Xicara)]);
        let res = calcular(&r, &als);
        assert!(perto(res.custo_total, 0.0));
        assert_eq!(res.avisos.len(), 1);
        assert!(validar_receita(&r, &als).is_err());
    }

    #[test]
    fn vd_da_porcao() {
        let als = vec![farinha()];
        let r = receita(vec![ing(1, 1000.0, Medida::G)]); // 3600 kcal / 4 porções = 900
        let res = calcular(&r, &als);
        assert!(perto(res.nutrientes[0].vd_porcao_pct.unwrap(), 45.0));
        assert!(res.nutrientes[2].vd_porcao_pct.is_none());
    }

    #[test]
    fn validacao_de_alimento() {
        let mut a = farinha();
        assert!(validar_alimento(&a).is_ok());
        a.qtd_embalagem = 0.0;
        assert!(validar_alimento(&a).is_err());
        a.qtd_embalagem = 1.0;
        a.nutrientes.sodio_mg = -1.0;
        assert!(validar_alimento(&a).is_err());
    }
}
