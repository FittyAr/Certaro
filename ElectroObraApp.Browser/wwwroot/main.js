import { dotnet } from './_framework/dotnet.js'

const is_browser = typeof window != "undefined";
if (!is_browser) throw new Error(`Expected to be running in a browser`);

function hideSplash() {
    document.querySelector('.avalonia-splash')?.remove();
}

function showStartupError(error) {
    const splash = document.querySelector('.avalonia-splash');
    if (!splash) return;

    splash.style.pointerEvents = 'auto';
    splash.classList.remove('splash-close');
    splash.innerHTML = `
        <div class="splash-brand">
            <div class="splash-mark">!</div>
            <h1 class="splash-title">No se pudo iniciar</h1>
            <p class="splash-subtitle" style="max-width: 32rem; text-align: center; color: #8B2E2E;">
                ${error}
            </p>
        </div>`;
}

try {
    const dotnetRuntime = await dotnet
        .withDiagnosticTracing(false)
        .withApplicationArgumentsFromQuery()
        .create();

    const config = dotnetRuntime.getConfig();
    await dotnetRuntime.runMain(config.mainAssemblyName, [globalThis.location.href]);

    hideSplash();

    const observer = new MutationObserver(() => {
        if (document.querySelector('canvas.avalonia-canvas')) {
            hideSplash();
            observer.disconnect();
        }
    });
    observer.observe(document.getElementById('out') ?? document.body, { childList: true, subtree: true });
} catch (error) {
    console.error('ElectroObra browser startup failed:', error);
    showStartupError(error?.message ?? String(error));
}
