# Plano de Aperfeiçoamento do ruwall

## Análise Comparativa: ruwall vs pywal

### Funcionalidades do pywal que faltam no ruwall:

1. **Múltiplos backends de geração de cores** (crítico)
   - pywal: wal, haishoku, colorz, colorthief, schemer2, fast_colorthief, modern_colorthief
   - ruwall: apenas wal (kmeans)

2. **Opção `--fg` (foreground)** - definir cor de foreground personalizada

3. **Opção `--cols16`** - método darken/lighten para 16 cores

4. **Mais templates de exportação**:
   - colors-oomox, colors.styl, colors-themer.js, colors-tilix.json, colors-wal.vim

5. **Melhor sistema de cache** - versionamento mais robusto

6. **Suporte a mais formatos de imagem** - webp, bmp, etc.

7. **Opção `--backend random`** - escolha aleatória de backend

8. **Melhor tratamento de erros** - mensagens mais informativas

9. **Opção `-g` (generate only)** - apenas gerar sem aplicar

## Implementação

### Fase 1: Backends de Cores Adicionais
- [ ] Implementar backend "colorz" (kmeans com ajuste vibrante)
- [ ] Implementar backend "colorthief" (median cut)
- [ ] Implementar backend "haishoku" (frequência de cores)
- [ ] Atualizar `colors::list_backends()` para listar todos
- [ ] Atualizar `colors::get_backend()` para suportar "random"

### Fase 2: Opções CLI Adicionais
- [ ] Adicionar opção `--fg` para foreground personalizado
- [ ] Adicionar opção `--cols16` com método darken/lighten
- [ ] Adicionar opção `-g` (generate only)

### Fase 3: Templates Adicionais
- [ ] Adicionar colors-oomox
- [ ] Adicionar colors.styl
- [ ] Adicionar colors-themer.js
- [ ] Adicionar colors-tilix.json
- [ ] Adicionar colors-wal.vim

### Fase 4: Otimizações
- [ ] Otimizar algoritmo kmeans (usar estruturas de dados mais eficientes)
- [ ] Melhorar sistema de cache
- [ ] Adicionar suporte a mais formatos de imagem
- [ ] Melhorar mensagens de erro

### Fase 5: Testes
- [ ] Adicionar testes para novos backends
- [ ] Adicionar testes para novas opções CLI
- [ ] Adicionar testes para novos templates
