# Ponte aura-dl para Firefox e LibreWolf

Adiciona "Baixar com aura-dl" ao menu de contexto do navegador. A extensÃ£o nÃ£o executa nada: ela envia a URL para o host nativo `aura-dl-host`, que valida tudo e chama `aura-dl --cli`.

## PeÃ§as

- `manifest.json` e `background.js`: extensÃ£o (menu de contexto e notificaÃ§Ãµes).
- `aura-dl-host`: host nativo em Python, que lÃª mensagens do navegador pelo protocolo de mensagens nativas.
- `aura_dl.json`: manifesto do host, que diz ao navegador onde ele estÃ¡ e quais extensÃµes podem usÃ¡-lo.

Na ISO, o host e o manifesto sÃ£o instalados em `/usr/local/bin/aura-dl-host` e `/usr/lib/mozilla/native-messaging-hosts/aura_dl.json`.

## LimitaÃ§Ãµes

- **Assinatura da extensÃ£o:** o Firefox release recusa extensÃµes nÃ£o assinadas pela Mozilla. Para instalar de forma permanente, a extensÃ£o precisa ser assinada no AMO. Para testar agora, use `about:debugging` > "Este Firefox" > "Carregar extensÃ£o temporÃ¡ria" e escolha o `manifest.json`.
- **LibreWolf:** o LibreWolf lÃª o manifesto de host de outro caminho, que nÃ£o foi verificado nesta versÃ£o. Se a ponte nÃ£o funcionar no LibreWolf, copie `aura_dl.json` para o diretÃ³rio de mensagens nativas dele.
- **aura-dl ausente:** o host responde com erro claro se o binÃ¡rio nÃ£o estiver no PATH. O `aura-dl` ainda nÃ£o entra na ISO.
- **NotificaÃ§Ãµes:** o `browser.notifications` exige a permissÃ£o `notifications`, jÃ¡ declarada no manifesto.

## Teste manual do host

```bash
printf '\x30\x00\x00\x00{"url":"https://exemplo.com/v","mode":"video"}' | python3 aura-dl-host
```

A resposta esperada Ã© um JSON com `ok` e, se algo falhar, `error`.

