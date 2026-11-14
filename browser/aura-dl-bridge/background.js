// Menu de contexto "Baixar com aura-dl" para links e para a própria página.
// A extensão só monta a mensagem; quem valida e executa é o host nativo.

const HOST_NAME = "aura_dl";

browser.menus.create({
  id: "aura-dl-link",
  title: "Baixar link com aura-dl",
  contexts: ["link"],
});

browser.menus.create({
  id: "aura-dl-page",
  title: "Baixar esta página com aura-dl",
  contexts: ["page"],
});

// Assim que o usuário clica, mandamos a URL ao host e mostramos o resultado.
browser.menus.onClicked.addListener(async (info, tab) => {
  const url = info.linkUrl || info.pageUrl || (tab && tab.url);
  if (!url) {
    notify("Não encontrei uma URL para baixar.");
    return;
  }

  try {
    const response = await browser.runtime.sendNativeMessage(HOST_NAME, {
      url,
      mode: "video",
      container: "mp4",
      quality: 1080,
    });
    if (response && response.ok) {
      notify("Download concluído.");
    } else {
      notify(`Falhou: ${response ? response.error : "sem resposta do host"}`);
    }
  } catch (err) {
    // Sem o host instalado, o Firefox lança erro aqui; a mensagem orienta o usuário.
    notify("Host nativo não encontrado. Instale o pacote aura-dl e a ponte.");
  }
});

function notify(message) {
  browser.notifications.create({
    type: "basic",
    title: "aura-dl",
    message,
  });
}
