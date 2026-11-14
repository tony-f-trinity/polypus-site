//! Formato de arquivo, dados de exemplo e números no jeito brasileiro.

use crate::modelo::*;

/// Lê o JSON salvo, aceitando versões antigas e conferindo ids repetidos e referências.
pub fn ler(texto: &str) -> Result<Dados, String> {
    let mut d: Dados = serde_json::from_str(texto).map_err(|e| format!("O arquivo não é um caderno de receitas válido ({e})."))?;
    if d.versao > VERSAO_FORMATO {
        return Err("Este arquivo foi feito por uma versão mais nova da ferramenta. Atualize a página.".into());
    }
    d.versao = VERSAO_FORMATO;
    let mut vistos = std::collections::HashSet::new();
    for a in &d.alimentos {
        if a.id == 0 || !vistos.insert(a.id) {
            return Err(format!("O alimento \"{}\" está com id repetido ou vazio no arquivo.", a.nome));
        }
        crate::calculo::validar_alimento(a).map_err(|e| format!("Alimento \"{}\": {e}", a.nome))?;
    }
    vistos.clear();
    for r in &d.receitas {
        if r.id == 0 || !vistos.insert(r.id) {
            return Err(format!("A receita \"{}\" está com id repetido ou vazio no arquivo.", r.nome));
        }
        crate::calculo::validar_receita(r, &d.alimentos).map_err(|e| format!("Receita \"{}\": {e}", r.nome))?;
    }
    Ok(d)
}

/// Chave para ordenar e comparar nomes sem ligar para acentos e maiúsculas.
pub fn chave_ordem(s: &str) -> String {
    s.trim()
        .chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            c => c,
        })
        .collect()
}

pub fn numero(v: f64, casas: usize, enxuto: bool) -> String {
    if !v.is_finite() {
        return "–".into();
    }
    let s = format!("{:.*}", casas, v.abs());
    let (int, dec) = match s.split_once('.') {
        Some((i, d)) => (i.to_string(), d.to_string()),
        None => (s.clone(), String::new()),
    };
    let mut agrupado = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            agrupado.push('.');
        }
        agrupado.push(c);
    }
    let dec = if enxuto { dec.trim_end_matches('0').to_string() } else { dec };
    let zero = agrupado.chars().all(|c| c == '0' || c == '.') && dec.chars().all(|c| c == '0');
    let sinal = if v < 0.0 && !zero { "-" } else { "" };
    if dec.is_empty() {
        format!("{sinal}{agrupado}")
    } else {
        format!("{sinal}{agrupado},{dec}")
    }
}

pub fn moeda(v: f64) -> String {
    let n = numero(v, 2, false);
    match n.strip_prefix('-') {
        Some(resto) => format!("-R$ {resto}"),
        None => format!("R$ {n}"),
    }
}

/// "1.234,5" → 1234.5 · "0,25" → 0.25 · "2.5" → 2.5 · "1.000" → 1000 · "R$ 3,90" → 3.9
pub fn ler_numero(texto: &str) -> Option<f64> {
    let t: String = texto
        .trim()
        .trim_start_matches("R$")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if t.is_empty() {
        return None;
    }
    let normal = if t.contains(',') {
        // vírgula é o decimal; pontos são separadores de milhar
        t.replace('.', "").replace(',', ".")
    } else if milhar_com_ponto(&t) {
        t.replace('.', "")
    } else {
        t
    };
    normal.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// "1.000" e "12.500.000" são milhares; "0.500", "2.5" e "1.25" são decimais.
fn milhar_com_ponto(t: &str) -> bool {
    let corpo = t.strip_prefix('-').unwrap_or(t);
    let partes: Vec<&str> = corpo.split('.').collect();
    partes.len() >= 2
        && (1..=3).contains(&partes[0].len())
        && !partes[0].starts_with('0')
        && partes.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
        && partes[1..].iter().all(|p| p.len() == 3)
}

/// Alguns alimentos com valores aproximados (tabela TACO e rótulos comuns) e preços de referência.
pub fn exemplo() -> Dados {
    fn al(id: u32, nome: &str, unidade: Unidade, preco: f64, qtd: f64, base: f64, n: [f64; 8], g: Option<f64>) -> Alimento {
        Alimento {
            id,
            nome: nome.into(),
            unidade,
            preco,
            qtd_embalagem: qtd,
            base_nutricional: base,
            nutrientes: Nutrientes {
                energia_kcal: n[0],
                carboidratos_g: n[1],
                acucares_g: n[2],
                proteinas_g: n[3],
                gorduras_totais_g: n[4],
                gorduras_saturadas_g: n[5],
                fibras_g: n[6],
                sodio_mg: n[7],
            },
            gramas_por_unidade: g,
            obs: "Valores de exemplo. Confira a embalagem do seu produto.".into(),
        }
    }
    use Unidade::*;
    //                                                                       kcal   carb  açúc  prot  gord  sat   fibra sódio
    let alimentos = vec![
        al(1, "Farinha de trigo", G, 5.49, 1000.0, 100.0, [360.0, 75.1, 0.3, 9.8, 1.4, 0.3, 2.3, 1.0], None),
        al(2, "Açúcar refinado", G, 4.99, 1000.0, 100.0, [387.0, 99.5, 99.5, 0.3, 0.0, 0.0, 0.0, 12.0], None),
        al(3, "Ovo", Un, 13.90, 12.0, 1.0, [72.0, 0.4, 0.2, 6.3, 4.8, 1.6, 0.0, 70.0], Some(50.0)),
        al(4, "Leite integral", Ml, 5.29, 1000.0, 200.0, [122.0, 9.4, 9.4, 6.4, 6.6, 4.2, 0.0, 100.0], Some(1.03)),
        al(5, "Óleo de soja", Ml, 7.99, 900.0, 100.0, [828.0, 0.0, 0.0, 0.0, 92.0, 14.4, 0.0, 0.0], Some(0.92)),
        al(6, "Chocolate em pó 50%", G, 9.90, 200.0, 100.0, [400.0, 70.0, 50.0, 6.0, 8.0, 4.5, 8.0, 40.0], None),
        al(7, "Fermento químico", G, 4.50, 100.0, 100.0, [90.0, 22.0, 0.0, 0.0, 0.0, 0.0, 0.0, 10000.0], None),
        al(8, "Manteiga com sal", G, 12.90, 200.0, 100.0, [726.0, 0.1, 0.1, 0.4, 82.4, 48.0, 0.0, 579.0], None),
    ];
    let ing = |id, quantidade, medida| Ingrediente { alimento_id: id, quantidade, medida };
    let receitas = vec![Receita {
        id: 1,
        nome: "Bolo de chocolate".into(),
        porcoes: 12.0,
        peso_final_g: Some(950.0),
        margem_pct: Some(150.0),
        ingredientes: vec![
            ing(1, 240.0, Medida::G),
            ing(2, 200.0, Medida::G),
            ing(3, 3.0, Medida::Un),
            ing(4, 1.0, Medida::Xicara),
            ing(5, 120.0, Medida::Ml),
            ing(6, 60.0, Medida::G),
            ing(7, 15.0, Medida::G),
        ],
        preparo: "Bata os ovos, o açúcar e o óleo. Junte o leite, a farinha e o chocolate. Por último, o fermento. Asse em forno médio por cerca de 40 minutos.".into(),
    }];
    Dados { versao: VERSAO_FORMATO, alimentos, receitas }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn numeros_brasileiros() {
        assert_eq!(numero(1234.5, 2, false), "1.234,50");
        assert_eq!(numero(1234.5, 2, true), "1.234,5");
        assert_eq!(numero(1000000.0, 0, false), "1.000.000");
        assert_eq!(numero(-0.001, 2, false), "0,00");
        assert_eq!(moeda(-3.5), "-R$ 3,50");
        assert_eq!(moeda(0.0), "R$ 0,00");
    }

    #[test]
    fn leitura() {
        assert_eq!(ler_numero("1.234,5"), Some(1234.5));
        assert_eq!(ler_numero("0,25"), Some(0.25));
        assert_eq!(ler_numero("2.5"), Some(2.5));
        assert_eq!(ler_numero("1.000"), Some(1000.0));
        assert_eq!(ler_numero("0.500"), Some(0.5));
        assert_eq!(ler_numero("R$ 3,90"), Some(3.9));
        assert_eq!(ler_numero(""), None);
        assert_eq!(ler_numero("abc"), None);
    }

    #[test]
    fn ordem_sem_acento() {
        assert_eq!(chave_ordem(" Óleo "), "oleo");
    }
}
