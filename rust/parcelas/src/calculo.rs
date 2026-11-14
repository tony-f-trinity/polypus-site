//! A conta: pagar à vista ou deixar o dinheiro no CDI e resgatar a parcela todo mês.
//!
//! - Rendimento: CDI anual convertido para taxa diária (252 dias úteis) × % do CDI,
//!   com 21 dias úteis por mês.
//! - Cada parcela é paga com um resgate. O IR incide só sobre a parte do resgate que é
//!   rendimento, na proporção rendimento/saldo, com a alíquota regressiva pelo prazo do
//!   resgate (30 dias por mês).
//! - Sem IOF: os resgates acontecem a partir de 30 dias, ou no ato, antes de render.
//!
//! Valores em reais (`f64`); a formatação em centavos fica em `lib.rs`.

/// Abaixo disso (meio centavo) a diferença é tratada como zero.
pub const TOLERANCIA: f64 = 0.005;

/// Alíquota do IR regressivo pelo número de dias corridos da aplicação.
pub fn aliquota_ir(dias: u32) -> f64 {
    match dias {
        0..=180 => 0.225,
        181..=360 => 0.20,
        361..=720 => 0.175,
        _ => 0.15,
    }
}

/// Rendimento bruto de um mês. `cdi_anual` e `pct_cdi` como fração (0,149 e 1,0).
pub fn taxa_mensal(cdi_anual: f64, pct_cdi: f64) -> f64 {
    let diaria = (1.0 + cdi_anual.max(0.0)).powf(1.0 / 252.0) - 1.0;
    (1.0 + diaria * pct_cdi.max(0.0)).powi(21) - 1.0
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mes {
    /// 0 = no ato da compra.
    pub mes: u32,
    pub dias: u32,
    pub saldo_antes: f64,
    pub rendimento: f64,
    pub resgate: f64,
    pub ir: f64,
    /// Quanto da parcela o resgate cobriu (menos que a parcela quando o dinheiro acaba).
    pub pago: f64,
    pub saldo_depois: f64,
    pub faltou: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Simulacao {
    pub meses: Vec<Mes>,
    /// Soma do que não deu para pagar.
    pub falta: f64,
    /// O que sobra depois da última parcela, já descontado o IR do resgate final.
    pub sobra_liquida: f64,
}

impl Simulacao {
    pub fn cobre(&self) -> bool {
        self.falta < TOLERANCIA
    }
}

/// Mês de cada parcela: 1, 2, 3… ou 0, 1, 2… quando a primeira é no ato.
fn mes_da_parcela(k: u32, no_ato: bool) -> u32 {
    if no_ato { k } else { k + 1 }
}

/// Aplica `aplicado` hoje e resgata, a cada parcela, o bruto que deixa a parcela líquida de IR.
pub fn simular(aplicado: f64, vezes: u32, parcela: f64, no_ato: bool, taxa: f64) -> Simulacao {
    let mut saldo = aplicado.max(0.0);
    let mut principal = saldo;
    let mut anterior = 0;
    let mut falta = 0.0;
    let mut ultimo_dias = 0;
    let mut meses = Vec::with_capacity(vezes as usize);
    for k in 0..vezes {
        let mes = mes_da_parcela(k, no_ato);
        let dias = mes * 30;
        let saldo_antes = saldo;
        saldo *= (1.0 + taxa).powi((mes - anterior) as i32);
        anterior = mes;
        let rendimento = saldo - saldo_antes;
        let t = aliquota_ir(dias);
        let fatia_rendimento = if saldo > 0.0 { ((saldo - principal) / saldo).max(0.0) } else { 0.0 };
        let mut resgate = parcela / (1.0 - t * fatia_rendimento);
        let (ir, pago, faltou);
        if resgate > saldo + 1e-9 {
            ir = t * (saldo - principal).max(0.0);
            pago = (saldo - ir).max(0.0);
            resgate = saldo;
            falta += parcela - pago;
            faltou = true;
            saldo = 0.0;
            principal = 0.0;
        } else {
            ir = resgate * fatia_rendimento * t;
            pago = parcela;
            faltou = false;
            principal -= resgate * (1.0 - fatia_rendimento);
            saldo -= resgate;
        }
        ultimo_dias = dias;
        meses.push(Mes { mes, dias, saldo_antes, rendimento, resgate, ir, pago, saldo_depois: saldo, faltou });
    }
    let sobra_liquida = saldo - aliquota_ir(ultimo_dias) * (saldo - principal).max(0.0);
    Simulacao { meses, falta, sobra_liquida }
}

/// Menor valor que, aplicado hoje, paga todas as parcelas.
pub fn necessario_hoje(vezes: u32, parcela: f64, no_ato: bool, taxa: f64) -> f64 {
    let (mut baixo, mut alto) = (0.0, vezes as f64 * parcela);
    for _ in 0..80 {
        let meio = (baixo + alto) / 2.0;
        if simular(meio, vezes, parcela, no_ato, taxa).cobre() {
            alto = meio;
        } else {
            baixo = meio;
        }
    }
    alto
}

/// Juros mensal embutido no parcelamento: a taxa `i` em que
/// à vista = soma de parcela / (1 + i)^mês. `None` quando não há solução na faixa.
pub fn juros_embutido(vista: f64, vezes: u32, parcela: f64, no_ato: bool) -> Option<f64> {
    let valor_presente = |i: f64| -> f64 {
        (0..vezes).map(|k| parcela / (1.0 + i).powi(mes_da_parcela(k, no_ato) as i32)).sum()
    };
    if (valor_presente(0.0) - vista).abs() < 1e-6 {
        return Some(0.0);
    }
    let (mut baixo, mut alto) = (-0.5, 2.0);
    if (valor_presente(baixo) - vista) * (valor_presente(alto) - vista) > 0.0 {
        return None;
    }
    for _ in 0..200 {
        let meio = (baixo + alto) / 2.0;
        if valor_presente(meio) > vista {
            baixo = meio;
        } else {
            alto = meio;
        }
    }
    Some((baixo + alto) / 2.0)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn perto(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn faixas_do_ir() {
        assert_eq!(aliquota_ir(0), 0.225);
        assert_eq!(aliquota_ir(180), 0.225);
        assert_eq!(aliquota_ir(181), 0.20);
        assert_eq!(aliquota_ir(360), 0.20);
        assert_eq!(aliquota_ir(720), 0.175);
        assert_eq!(aliquota_ir(721), 0.15);
    }

    #[test]
    fn taxa_do_mes() {
        assert!(perto(taxa_mensal(0.149, 1.0), 0.011641575, 1e-8));
        assert!(taxa_mensal(0.149, 1.1) > taxa_mensal(0.149, 1.0));
        assert_eq!(taxa_mensal(0.0, 1.0), 0.0);
    }

    #[test]
    fn sem_rendimento_precisa_do_total() {
        let x = necessario_hoje(10, 100.0, false, 0.0);
        assert!(perto(x, 1000.0, 0.01));
        assert_eq!(juros_embutido(1000.0, 10, 100.0, false), Some(0.0));
    }

    #[test]
    fn aplicar_o_necessario_zera_o_saldo() {
        let m = taxa_mensal(0.149, 1.0);
        let x = necessario_hoje(10, 100.0, false, m);
        let s = simular(x, 10, 100.0, false, m);
        assert!(s.cobre());
        assert!(s.sobra_liquida.abs() < 0.01);
        // valores conferidos na versão em JavaScript
        assert!(perto(x, 950.88, 0.01));
        assert!(perto(juros_embutido(950.0, 10, 100.0, false).unwrap(), 0.00944, 0.00001));
    }

    #[test]
    fn ir_so_sobre_o_rendimento() {
        let m = 0.01;
        let s = simular(1000.0, 1, 100.0, false, m);
        let mes = &s.meses[0];
        // saldo 1010, rendimento 10 → fatia 10/1010, alíquota 22,5%
        let fatia = 10.0 / 1010.0;
        assert!(perto(mes.resgate, 100.0 / (1.0 - 0.225 * fatia), 1e-9));
        assert!(perto(mes.resgate - mes.ir, 100.0, 1e-9));
    }

    #[test]
    fn no_ato_nao_paga_ir() {
        let s = simular(500.0, 1, 100.0, true, 0.01);
        assert_eq!(s.meses[0].ir, 0.0);
        assert_eq!(s.meses[0].mes, 0);
    }

    #[test]
    fn aponta_o_que_falta() {
        let s = simular(150.0, 3, 100.0, false, 0.0);
        assert!(!s.cobre());
        assert!(perto(s.falta, 150.0, 1e-9));
        assert!(s.meses[1].faltou && s.meses[2].faltou);
    }
}
