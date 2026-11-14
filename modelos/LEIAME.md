# Modelos 3D dos projetos

Coloque aqui o arquivo 3D de cada projeto e aponte para ele no campo `modelo` em `js/dados.js`.

- Formato: **.glb** (recomendado, um arquivo só) ou .gltf.
- Exportar do Blender: Arquivo > Exportar > glTF 2.0 (.glb). Do Fusion 360, Onshape ou SolidWorks,
  exporte em .obj/.stl/.step e converta para .glb (o Blender abre esses formatos).
- Tamanho: tente ficar abaixo de ~5 MB por modelo para a página abrir rápido.
- Opcional: uma imagem (.jpg/.png/.webp) no campo `imagem`, que aparece enquanto o modelo carrega.

Exemplo:
    { situacao: 'Rascunho', nome: 'Walkman', ..., modelo: 'modelos/walkman.glb', imagem: 'modelos/walkman.jpg' }
