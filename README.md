# Polypus

Site estático do Polypus, com três abas: **Início**, **Ferramentas** e **Projetos**.
Feito no mesmo molde do Tempo Continuum, com paleta vermelha e tema claro/escuro.

## Estrutura

```
index.html          Início (aba principal)
ferramentas.html    Ferramentas, com busca e filtro "Do Polypus / Recomendadas"
projetos.html       Projetos, com busca e filtro por situação
css/polypus.css     Cores, fontes e layout de todas as páginas
js/dados.js         Conteúdo: lista de ferramentas e de projetos
js/polypus.js       Tema, menu do celular, filtros e montagem das listas
assets/polypus.svg  Logo (também usado como ícone da aba)
.nojekyll           Faz o GitHub Pages servir os arquivos como estão
```

## Como editar o conteúdo

Abra `js/dados.js`. Cada ferramenta e cada projeto é uma linha da lista.
Troque os textos entre colchetes, apague os exemplos que não usar e copie uma linha para criar outra.

- Ferramenta com `link: ''` aparece como **Em breve**.
- `destaque: true` coloca o item na página inicial (até 3 ferramentas e 4 projetos).
- Situações de projeto aceitas: `'Em andamento'`, `'Planejado'` e `'Concluído'`.

## Cores

| Papel | Claro | Escuro |
|---|---|---|
| Fundo | `#F7F0EC` | `#140708` |
| Cartões | `#EDE0DA` | `#2B1819` |
| Texto | `#160809` | `#F7F0EC` |
| Texto secundário | `#4D3739` | `#DCC9C4` |
| Vermelho (destaques e botões) | `#C41E2A` | `#C41E2A` |
| Texto sobre o vermelho | `#FFF6F3` | `#FFF6F3` |

As cores ficam no topo de `css/polypus.css`. A fonte é Gotham Medium quando instalada; senão, Montserrat Medium.

## Publicar no GitHub Pages

1. Crie um repositório público (por exemplo, `polypus`) na conta onde o site vai ficar.
2. Envie todos os arquivos desta pasta para a raiz do repositório, incluindo `.nojekyll`.
3. No repositório, vá em **Settings → Pages**, em **Source** escolha **Deploy from a branch**, selecione a branch `main` e a pasta `/ (root)`, e salve.
4. Em um ou dois minutos o site fica no ar em `https://SEU-USUARIO.github.io/polypus/`.
