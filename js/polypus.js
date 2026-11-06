/* Polypus — tema claro/escuro, menu do celular e montagem das listas a partir de js/dados.js */
(function () {
  'use strict';
  var raiz = document.documentElement;
  var dados = window.POLYPUS || { ferramentas: [], projetos: [] };

  /* ---------- Tema ---------- */
  function escuroAgora() {
    var t = raiz.getAttribute('data-theme');
    if (t === 'dark') return true;
    if (t === 'light') return false;
    try { return window.matchMedia('(prefers-color-scheme: dark)').matches; } catch (e) { return false; }
  }
  function rotularTema() {
    var escuro = escuroAgora();
    document.querySelectorAll('[data-tema]').forEach(function (b) {
      b.setAttribute('aria-label', escuro ? 'Mudar para o tema claro' : 'Mudar para o tema escuro');
    });
  }
  document.querySelectorAll('[data-tema]').forEach(function (b) {
    b.addEventListener('click', function () {
      var novo = !escuroAgora();
      raiz.setAttribute('data-theme', novo ? 'dark' : 'light');
      try { window.localStorage.setItem('pp-tema', novo ? 'escuro' : 'claro'); } catch (e) {}
      rotularTema();
    });
  });
  try { window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', rotularTema); } catch (e) {}
  rotularTema();

  /* ---------- Menu do celular ---------- */
  var botaoMenu = document.querySelector('[data-menu]');
  var painel = document.getElementById('pp-painel');
  if (botaoMenu && painel) {
    botaoMenu.addEventListener('click', function () {
      var abrir = painel.hidden;
      painel.hidden = !abrir;
      botaoMenu.setAttribute('aria-expanded', abrir ? 'true' : 'false');
      botaoMenu.querySelector('.ic-abrir').hidden = abrir;
      botaoMenu.querySelector('.ic-fechar').hidden = !abrir;
    });
  }

  /* ---------- Ajudantes ---------- */
  function el(tag, cls, texto) {
    var n = document.createElement(tag);
    if (cls) n.className = cls;
    if (texto != null) n.textContent = texto;
    return n;
  }
  function norm(s) { return String(s || '').normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().trim(); }
  function casa(item, campos, palavras) {
    var t = norm(campos.map(function (c) { return item[c]; }).join(' '));
    return palavras.every(function (w) { return t.indexOf(w) >= 0; });
  }
  function externo(link) { return /^https?:\/\//.test(link || ''); }

  var SELO_SITUACAO = { 'Em andamento': 'selo-cheio', 'Planejado': 'selo-contorno', 'Concluído': 'selo-tracejado' };

  function cartaoFerramenta(f) {
    var a = el('article', 'cartao');
    var meta = el('div', 'cartao-meta');
    meta.appendChild(el('span', 'selo selo-cheio', 'Do Polypus'));
    meta.appendChild(el('span', 'braco', f.braco));
    a.appendChild(meta);
    a.appendChild(el('h3', null, f.nome));
    a.appendChild(el('p', null, f.descricao));
    var pe = el('div', 'rodape-cartao');
    if (f.link) {
      var b = el('a', 'btn', 'Abrir ferramenta');
      b.href = f.link;
      if (externo(f.link)) { b.target = '_blank'; b.rel = 'noopener noreferrer'; }
      pe.appendChild(b);
    } else {
      pe.appendChild(el('span', 'em-breve', 'Em breve'));
    }
    a.appendChild(pe);
    return a;
  }

  function linhaRecomendada(f) {
    var a = el('article', 'linha');
    var c1 = el('div', 'col-selo');
    c1.appendChild(el('span', 'selo selo-contorno', 'Recomendada'));
    c1.appendChild(el('span', 'lado', f.braco));
    a.appendChild(c1);
    var c2 = el('div', 'col-texto');
    c2.appendChild(el('h3', null, f.nome));
    c2.appendChild(el('p', null, f.descricao));
    if (f.quem) c2.appendChild(el('span', 'lado', 'Mantida por ' + f.quem));
    a.appendChild(c2);
    if (f.link) {
      var b = el('a', 'btn-sair', 'Acessar ↗');
      b.href = f.link; b.target = '_blank'; b.rel = 'noopener noreferrer';
      b.setAttribute('aria-label', 'Acessar ' + f.nome + ' (abre em nova aba)');
      a.appendChild(b);
    }
    return a;
  }

  function linhaProjeto(p) {
    var a = el('article', 'linha');
    var c1 = el('div', 'col-selo');
    c1.appendChild(el('span', 'selo ' + (SELO_SITUACAO[p.situacao] || 'selo-contorno'), p.situacao));
    a.appendChild(c1);
    var c2 = el('div', 'col-texto');
    c2.appendChild(el('h3', null, p.nome));
    if (p.descricao) c2.appendChild(el('p', null, p.descricao));
    var info = [p.braco];
    if (p.inicio) info.push((p.situacao === 'Planejado' ? 'Previsto para ' : 'Desde ') + p.inicio);
    if (p.atualizado) info.push('Atualizado em ' + p.atualizado);
    c2.appendChild(el('span', 'lado', info.join(' · ')));
    a.appendChild(c2);
    if (p.link) {
      var b = el('a', 'btn-sair', 'Ver projeto');
      b.href = p.link;
      if (externo(p.link)) { b.target = '_blank'; b.rel = 'noopener noreferrer'; }
      a.appendChild(b);
    }
    return a;
  }

  function encher(alvo, itens, fazer) {
    alvo.replaceChildren();
    itens.forEach(function (i) { alvo.appendChild(fazer(i)); });
  }

  function chips(alvo, opcoes, ativa, escolher) {
    alvo.replaceChildren();
    opcoes.forEach(function (o) {
      var b = el('button', 'chip', o.nome + ' · ' + o.n);
      b.type = 'button';
      b.setAttribute('aria-pressed', o.nome === ativa ? 'true' : 'false');
      b.addEventListener('click', function () { escolher(o.nome); });
      alvo.appendChild(b);
    });
  }

  var nossas = dados.ferramentas.filter(function (f) { return f.tipo === 'Nossa'; });
  var recomendadas = dados.ferramentas.filter(function (f) { return f.tipo !== 'Nossa'; });

  /* ---------- Início ---------- */
  var homeF = document.getElementById('home-ferramentas');
  if (homeF) {
    var dest = nossas.filter(function (f) { return f.destaque; }).slice(0, 3);
    encher(homeF, dest.length ? dest : nossas.slice(0, 3), cartaoFerramenta);
  }
  var homeP = document.getElementById('home-projetos');
  if (homeP) {
    var destP = dados.projetos.filter(function (p) { return p.destaque; }).slice(0, 4);
    encher(homeP, destP.length ? destP : dados.projetos.slice(0, 4), linhaProjeto);
  }
  document.querySelectorAll('[data-conta]').forEach(function (n) {
    var k = n.getAttribute('data-conta');
    var v = k === 'nossas' ? nossas.length
      : k === 'andamento' ? dados.projetos.filter(function (p) { return p.situacao === 'Em andamento'; }).length
      : k === 'projetos' ? dados.projetos.length : '';
    n.textContent = String(v);
  });

  /* ---------- Ferramentas ---------- */
  var listaNossas = document.getElementById('lista-nossas');
  if (listaNossas) {
    var estF = { q: '', tipo: 'Todas' };
    var campoF = document.getElementById('busca-ferramentas');
    var desenharF = function () {
      var palavras = norm(estF.q).split(/\s+/).filter(Boolean);
      var campos = ['nome', 'braco', 'descricao', 'quem'];
      var n = nossas.filter(function (f) { return casa(f, campos, palavras); });
      var r = recomendadas.filter(function (f) { return casa(f, campos, palavras); });
      chips(document.getElementById('chips-ferramentas'),
        [{ nome: 'Todas', n: n.length + r.length }, { nome: 'Do Polypus', n: n.length }, { nome: 'Recomendadas', n: r.length }],
        estF.tipo, function (t) { estF.tipo = t; desenharF(); });
      var verN = estF.tipo !== 'Recomendadas' && n.length > 0;
      var verR = estF.tipo !== 'Do Polypus' && r.length > 0;
      document.getElementById('sec-nossas').hidden = !verN;
      document.getElementById('sec-recomendadas').hidden = !verR;
      encher(listaNossas, n, cartaoFerramenta);
      encher(document.getElementById('lista-recomendadas'), r, linhaRecomendada);
      var total = (estF.tipo !== 'Recomendadas' ? n.length : 0) + (estF.tipo !== 'Do Polypus' ? r.length : 0);
      document.getElementById('vazio-ferramentas').hidden = total > 0;
      document.getElementById('conta-ferramentas').textContent = total + (total === 1 ? ' ferramenta' : ' ferramentas');
    };
    campoF.addEventListener('input', function (e) { estF.q = e.target.value; desenharF(); });
    estF.q = campoF.value || '';
    desenharF();
  }

  /* ---------- Projetos ---------- */
  var listaProj = document.getElementById('lista-projetos');
  if (listaProj) {
    var estP = { q: '', sit: 'Todos' };
    var MAPA = { 'Em andamento': 'Em andamento', 'Planejados': 'Planejado', 'Concluídos': 'Concluído' };
    var campoP = document.getElementById('busca-projetos');
    var desenharP = function () {
      var palavras = norm(estP.q).split(/\s+/).filter(Boolean);
      var base = dados.projetos.filter(function (p) { return casa(p, ['nome', 'braco', 'descricao', 'situacao'], palavras); });
      var conta = function (s) { return base.filter(function (p) { return p.situacao === s; }).length; };
      chips(document.getElementById('chips-projetos'),
        [{ nome: 'Todos', n: base.length }, { nome: 'Em andamento', n: conta('Em andamento') },
         { nome: 'Planejados', n: conta('Planejado') }, { nome: 'Concluídos', n: conta('Concluído') }],
        estP.sit, function (s) { estP.sit = s; desenharP(); });
      var lista = estP.sit === 'Todos' ? base : base.filter(function (p) { return p.situacao === MAPA[estP.sit]; });
      encher(listaProj, lista, linhaProjeto);
      document.getElementById('vazio-projetos').hidden = lista.length > 0;
      document.getElementById('conta-projetos').textContent = lista.length + (lista.length === 1 ? ' projeto' : ' projetos');
    };
    campoP.addEventListener('input', function (e) { estP.q = e.target.value; desenharP(); });
    estP.q = campoP.value || '';
    desenharP();
  }
})();
