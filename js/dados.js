/* Polypus — conteúdo do site.
   Este é o único arquivo que você precisa editar para adicionar ou mudar ferramentas e projetos.
   As páginas Início, Ferramentas e Projetos leem daqui.

   FERRAMENTAS
     tipo:      'Nossa' (feita pelo Polypus) ou 'Recomendada' (de outra organização)
     nome, braco (área), descricao
     link:      endereço da ferramenta. Deixe '' para mostrar "Em breve".
     quem:      quem mantém (usado nas recomendadas)
     destaque:  true para aparecer na página inicial

   PROJETOS
     situacao:  'Em andamento', 'Planejado' ou 'Concluído'
     nome, braco, descricao
     inicio:    texto livre, ex.: 'mar 2026'
     atualizado: texto livre, ex.: '06 nov 2026'
     link:      página do projeto ('' se ainda não tiver)
     destaque:  true para aparecer na página inicial
*/
window.POLYPUS = {
  ferramentas: [
    { tipo: 'Nossa', nome: '[Nome da primeira ferramenta]', braco: '[Braço]', descricao: '[Uma frase sobre o que a ferramenta faz e para quem.]', link: '#', quem: 'Polypus', destaque: true },
    { tipo: 'Nossa', nome: '[Nome da segunda ferramenta]', braco: '[Braço]', descricao: '[O problema que ela resolve, em uma ou duas frases.]', link: '#', quem: 'Polypus', destaque: true },
    { tipo: 'Nossa', nome: '[Nome da terceira ferramenta]', braco: '[Braço]', descricao: '[Para quem ela serve e como se usa.]', link: '', quem: 'Polypus', destaque: true },
    { tipo: 'Nossa', nome: '[Próxima ferramenta]', braco: '[Braço]', descricao: '[Uma frase sobre o que vem por aí.]', link: '', quem: 'Polypus', destaque: false },
    { tipo: 'Recomendada', nome: '[Ferramenta recomendada]', braco: '[Braço]', descricao: '[Por que recomendamos esta ferramenta, em uma frase.]', link: 'https://example.com', quem: '[Organização que mantém]' },
    { tipo: 'Recomendada', nome: '[Ferramenta recomendada]', braco: '[Braço]', descricao: '[O que ela faz e quando vale a pena usar.]', link: 'https://example.com', quem: '[Organização que mantém]' },
    { tipo: 'Recomendada', nome: '[Ferramenta recomendada]', braco: '[Braço]', descricao: '[Uma frase sobre a ferramenta.]', link: 'https://example.com', quem: '[Organização que mantém]' }
  ],

  projetos: [
    { situacao: 'Em andamento', nome: '[Título do projeto principal]', braco: '[Braço]', descricao: '[Duas frases sobre o objetivo do projeto e o que já foi feito.]', inicio: '[mês ano]', atualizado: '[data]', link: '#', destaque: true },
    { situacao: 'Em andamento', nome: '[Título do projeto]', braco: '[Braço]', descricao: '[O que o projeto quer resolver.]', inicio: '[mês ano]', atualizado: '[data]', link: '#', destaque: true },
    { situacao: 'Planejado', nome: '[Título do projeto]', braco: '[Braço]', descricao: '[O que está previsto e quando deve começar.]', inicio: '[previsão]', atualizado: '[data]', link: '', destaque: true },
    { situacao: 'Planejado', nome: '[Título do projeto]', braco: '[Braço]', descricao: '[Uma frase sobre a ideia.]', inicio: '[previsão]', atualizado: '[data]', link: '', destaque: false },
    { situacao: 'Concluído', nome: '[Título do projeto]', braco: '[Braço]', descricao: '[O que o projeto entregou.]', inicio: '[mês ano]', atualizado: '[data]', link: '#', destaque: true },
    { situacao: 'Concluído', nome: '[Título do projeto]', braco: '[Braço]', descricao: '[Resultado e onde encontrá-lo.]', inicio: '[mês ano]', atualizado: '[data]', link: '#', destaque: false }
  ]
};
