//! Dinheiro no formato brasileiro. Todos os valores são centavos inteiros (`i64`).

/// Maior valor aceito num campo: R$ 1 trilhão. Evita estouro nas contas.
pub const LIMITE: i64 = 100_000_000_000_000;

fn milhares(mut n: i64) -> String {
    let mut partes = Vec::new();
    loop {
        if n < 1000 {
            partes.push(n.to_string());
            break;
        }
        partes.push(format!("{:03}", n % 1000));
        n /= 1000;
    }
    partes.reverse();
    partes.join(".")
}

/// `123456` → `"1.234,56"`.
pub fn campo(c: i64) -> String {
    let sinal = if c < 0 { "-" } else { "" };
    let c = c.unsigned_abs() as i64;
    format!("{sinal}{},{:02}", milhares(c / 100), c % 100)
}

/// `123456` → `"R$ 1.234,56"`, com espaço estreito não separável (U+202F): o U+00A0
/// sai largo demais em algumas instalações da Gotham.
pub fn moeda(c: i64) -> String {
    if c < 0 {
        format!("-R$\u{202f}{}", campo(-c))
    } else {
        format!("R$\u{202f}{}", campo(c))
    }
}

/// `0.1234` → `"12,3%"`.
pub fn porcentagem(fracao: f64) -> String {
    let v = (fracao * 1000.0).round() as i64; // décimos de ponto percentual
    format!("{},{}%", v / 10, v % 10)
}

/// Lê um valor digitado: "1.234,56", "1234,56", "1234.56", "R$ 1.234", "1.234" (milhar).
/// Vazio, inválido, zero ou negativo → 0. Arredonda para o centavo mais próximo.
pub fn ler_centavos(texto: &str) -> i64 {
    let s: String = texto
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}' && *c != 'R' && *c != 'r' && *c != '$')
        .collect();
    if s.is_empty() || s.starts_with('-') {
        return 0;
    }
    let pontos = s.matches('.').count();
    let (inteira, fracao): (String, String) = if s.contains(',') {
        // vírgula é o decimal; pontos são milhar
        let mut it = s.splitn(2, ',');
        let i = it.next().unwrap_or("").replace('.', "");
        let f = it.next().unwrap_or("").to_string();
        (i, f)
    } else if pontos == 1 {
        let (i, f) = s.split_once('.').unwrap();
        if f.len() == 3 {
            // "1.234" em português é mil duzentos e trinta e quatro
            (format!("{i}{f}"), String::new())
        } else {
            (i.to_string(), f.to_string())
        }
    } else {
        (s.replace('.', ""), String::new())
    };
    if !inteira.chars().all(|c| c.is_ascii_digit()) || !fracao.chars().all(|c| c.is_ascii_digit()) {
        return 0;
    }
    let inteira = inteira.trim_start_matches('0');
    if inteira.len() > 13 {
        return LIMITE;
    }
    let reais: i64 = if inteira.is_empty() { 0 } else { inteira.parse().unwrap_or(0) };
    let d: Vec<i64> = fracao.bytes().take(3).map(|b| (b - b'0') as i64).collect();
    let mut cent = d.first().copied().unwrap_or(0) * 10 + d.get(1).copied().unwrap_or(0);
    if d.get(2).copied().unwrap_or(0) >= 5 {
        cent += 1;
    }
    (reais * 100 + cent).clamp(0, LIMITE)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn formata() {
        assert_eq!(campo(0), "0,00");
        assert_eq!(campo(5), "0,05");
        assert_eq!(campo(123456), "1.234,56");
        assert_eq!(campo(100_000_000), "1.000.000,00");
        assert_eq!(moeda(250000), "R$\u{202f}2.500,00");
        assert_eq!(porcentagem(0.1234), "12,3%");
        assert_eq!(porcentagem(1.0), "100,0%");
    }

    #[test]
    fn le() {
        assert_eq!(ler_centavos("1.234,56"), 123456);
        assert_eq!(ler_centavos("1234,56"), 123456);
        assert_eq!(ler_centavos("1234.56"), 123456);
        assert_eq!(ler_centavos("R$ 1.234"), 123400);
        assert_eq!(ler_centavos("1.234.567"), 123456700);
        assert_eq!(ler_centavos("0,5"), 50);
        assert_eq!(ler_centavos(",99"), 99);
        assert_eq!(ler_centavos("10,005"), 1001);
        assert_eq!(ler_centavos(""), 0);
        assert_eq!(ler_centavos("abc"), 0);
        assert_eq!(ler_centavos("-5"), 0);
        assert_eq!(ler_centavos("99999999999999999999"), LIMITE);
    }
}

/// Reais em ponto flutuante → centavos, arredondando.
pub fn centavos(reais: f64) -> i64 {
    if !reais.is_finite() {
        return 0;
    }
    (reais * 100.0).round().clamp(-(LIMITE as f64), LIMITE as f64) as i64
}

/// `0.020912` → `"2,09%"` (duas casas, aceita negativo).
pub fn porcentagem2(fracao: f64) -> String {
    format!("{}%", campo((fracao * 10000.0).round() as i64))
}

#[cfg(test)]
mod testes_extra {
    use super::*;

    #[test]
    fn converte() {
        assert_eq!(centavos(950.004), 95000);
        assert_eq!(centavos(-0.884), -88);
        assert_eq!(porcentagem2(0.020912), "2,09%");
        assert_eq!(porcentagem2(-0.0105), "-1,05%");
    }
}
