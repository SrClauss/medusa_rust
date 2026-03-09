# Instruções para Agente Autônomo

Você é um **agente Copilot autônomo** trabalhando no repositório `medusa_rust` localizado em `/home/claus/src/medusa_rust`.

Sua missão é completar o port do **Medusa JS v2 para Rust**. Para isso, siga rigorosamente as etapas abaixo e documente todo o progresso.

---

## PREPARAÇÃO INICIAL

1. Clone o repositório oficial do MedusaJS (apenas se ainda não existir dentro de `vendor/`):
   ```bash
   cd /home/claus/src/medusa_rust
   git clone https://github.com/medusajs/medusa.git vendor/medusa_official
   ```

2. Navegue pelo código do MedusaJS para entender como cada rota, serviço e utilitário está implementado. Preste atenção em:
   - Estrutura de rotas (store/admin)
   - Validação de payloads
   - Lógica de negócio nos serviços
   - Utilitários comuns (mapear, formatar, calcular)
   - Testes existentes (veja como as rotas são exercitadas)

3. Consulte a documentação oficial do Medusa (https://docs.medusajs.com) para garantir que nenhuma funcionalidade seja omitida ou adulterada. Use-a como referência para comportamento esperado, parâmetros e respostas JSON.

4. Mantenha o arquivo `IMPLEMENTATION_PLAN.md` atualizado com o status de cada rota, tabela de migrações e prioridades. Sempre que implementar uma rota ou funcionalidade, marque-a como ✅/🟡/❌ e anote qualquer observação relevante.

5. O repositório Rust já possui muitos exemplos de rotas e testes. Copie o padrão arquitetônico:
   - `src/api/*` para handlers
   - `src/core/*` para lógica de negócio
   - `src/storagea/*` para abstrações de banco/cache
   - `tests/*.rs` para suítes de integração
   - Use Axum para definir rotas e middleware, SQLx para acesso ao PostgreSQL, Moka para cache e MinIO/S3 para armazenamento de arquivos.

6. Priorize a implementação conforme as fases definidas no próprio `IMPLEMENTATION_PLAN.md`:
   - Fase 1 (Core Commerce): estoque, gift cards, devoluções, price lists, moedas
   - Fase 2 (Operações): rascunhos, edições, trocas, reclamações, canais de venda
   - Fase 3 (Extensibilidade): plugins, eventos, webhooks, chaves de API
   - Fase 4 (Integrações): provedores de pagamento/envio, busca, e-mail, OAuth
   - Fase 5 (Enterprise): multi-store, workflows, traduções, créditos, analytics

7. Crie migrações SQL sempre que uma nova tabela ou coluna for necessária. Coloque os arquivos sob `migrations/` seguindo o padrão de timestamp.

8. Escreva testes automatizados para cada rota/serviço implementado. Mantenha cobertura alta e evite regressões. Execute `cargo test` frequentemente.

9. Use o estilo de código do projeto (Clippy, format) e realize `cargo fmt` antes de commits. Siga as convenções já estabelecidas nos arquivos existentes.

10. Periodicamente, compile e rode o servidor (`cargo run`) para garantir que as alterações não quebrem o build e que as rotas funcionem em conjunto.

---

> **Nota:** Este documento serve como o guia mestre para suas ações. Sempre que tiver dúvidas, retorne ao código original do MedusaJS ou à documentação para confirmar comportamentos. Mantenha os logs de progresso e as marcações claras no `IMPLEMENTATION_PLAN.md` para que qualquer colaborador possa entender o estado atual do port.

Boa sorte, agente. O portamento completo depende do seu rigor e atenção aos detalhes.