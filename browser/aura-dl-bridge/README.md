# Ponte aura-dl para Firefox e LibreWolf

Adiciona "Baixar com aura-dl" ao menu de contexto do navegador. A extensão não
executa nada: ela envia a URL para o host nativo `aura-dl-host`, que valida
tudo e chama `aura-dl --cli`.

## Peças

- `manifest.json` e `background.js`: extensão (menu de contexto e notificações).
- `aura-dl-host`: host nativo em Python, que lê mensagens do navegador pelo
  protocolo de mensagens nativas.
- `aura_dl.json`: manifesto do host, que diz ao navegador onde ele está e
  quais extensões podem usá-lo.

Na ISO, o pacote `aura-dl-bridge` instala:

- o host em `/usr/bin/aura-dl-host`;
- o manifesto nativo para Firefox em
  `/usr/lib/mozilla/native-messaging-hosts/aura_dl.json`;
- o mesmo manifesto para LibreWolf em
  `/usr/lib/librewolf/native-messaging-hosts/aura_dl.json`;
- a extensão em `/usr/share/aura-os/aura-dl-bridge/`.

## Limitações

- **Assinatura da extensão:** o Firefox release recusa extensões não assinadas
  pela Mozilla. Para instalar de forma permanente, a extensão precisa ser
  assinada no AMO. Para testar agora, use `about:debugging` > "Este Firefox" >
  "Carregar extensão temporária" e escolha
  `/usr/share/aura-os/aura-dl-bridge/manifest.json`.
- **Notificações:** o `browser.notifications` exige a permissão `notifications`,
  já declarada no manifesto.

## Testes

```bash
python3 -m unittest discover -s tests -v
```

## Teste manual do host

```bash
printf '\x30\x00\x00\x00{"url":"https://exemplo.com/v","mode":"video"}' | python3 aura-dl-host
```

A resposta esperada é um JSON com `ok` e, se algo falhar, `error`.
