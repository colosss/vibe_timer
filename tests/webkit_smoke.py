"""WebKitGTK smoke test with mocked Tauri IPC (never runs power commands).

Run `npm run dev -- --host 127.0.0.1`, then
`/usr/bin/python3 tests/webkit_smoke.py` in a graphical session.
Requires PyGObject and WebKit2 4.1 introspection bindings.
"""
import gi
gi.require_version('Gtk','3.0'); gi.require_version('WebKit2','4.1')
from gi.repository import Gtk, WebKit2, GLib
mock = r'''
window.onerror=(m)=>window.webkit.messageHandlers.result.postMessage('JS ERROR '+m); window.callbacks = {}; window.listeners = {}; let id = 0;
window.__TAURI_INTERNALS__ = {
 transformCallback(cb) { callbacks[++id] = cb; return id; },
 unregisterCallback(i) { delete callbacks[i]; },
 invoke: async (cmd, args) => {
 if(cmd === 'plugin:event|listen') { listeners[args.event] = callbacks[args.handler]; return ++id; }
 if(cmd === 'load_settings') return {window_opacity:95,default_sound_volume:0,default_notification:'notification',always_on_top:false,minimize_to_tray:true,start_minimized:false};
 if(cmd === 'load_presets') return [{name:'Test \u{1F5D1}\uFE0F \u{1F50C}',duration:3600,action:'sleep',warning_time:30,notification_type:'notification',sound_volume:0}];
 return null;
 }
};
window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {unregisterListener() {}};
window.emitTest = (event,payload) => listeners[event]({event,payload});
'''
manager = WebKit2.UserContentManager()
manager.add_script(WebKit2.UserScript.new(mock,WebKit2.UserContentInjectedFrames.ALL_FRAMES,WebKit2.UserScriptInjectionTime.START,None,None))
view = WebKit2.WebView.new_with_user_content_manager(manager)
win = Gtk.Window(); win.set_default_size(480,720); win.add(view); win.show_all()
result = 1
checks = r'''
(async () => {
 const wait = ms => new Promise(r => setTimeout(r,ms));
 const assert = (v,m) => { if(!v) throw Error(m); };
 const click = text => { const b=[...document.querySelectorAll('button')].find(b=>b.textContent.includes(text)); assert(b,'missing '+text); b.click(); };
 for(let i=0;i<8;i++) {
 document.querySelector('[title="Пресеты"]').click(); await wait(80);
 assert(document.querySelector('.presets-modal'),'presets');
 assert(document.querySelector('.presets-header svg'),'SVG presets icon');
 if(i===0) {
 document.querySelector('[title="Редактировать"]').click(); await wait(80);
 assert(document.querySelector('.preset-edit-form'),'edit preset');
 click('Отмена'); await wait(80);
 click('Добавить пресет'); await wait(80);
 assert(document.querySelector('.preset-edit-form'),'add preset');
 click('Отмена'); await wait(80);
 }
 click('Применить'); await wait(80);
 assert(!document.querySelector('.presets-modal'),'close presets');
 }
 document.querySelector('[title="Настройки"]').click(); await wait(100);
 click('Протестировать'); await wait(100);
 assert(document.querySelector('.notification-preview'),'preview');
 document.querySelector('[title="Закрыть"]').click(); await wait(100);
 click('Запустить таймер'); await wait(100);
 emitTest('timer_warning',30); await wait(100);
 assert(document.querySelector('.warning-modal'),'warning');
 assert(document.querySelector('.warning-icon svg'),'SVG warning icon');
 emitTest('timer_tick',29); await wait(100);
 assert(document.querySelector('.warning-modal').textContent.includes('29'),'warning update');
 await wait(1200);
 assert(document.querySelector('.warning-modal').textContent.includes('29'),'paused countdown drift');
 emitTest('timer_tick',28); await wait(100);
 assert(document.querySelector('.warning-modal').textContent.includes('28'),'countdown resume');
 document.querySelector('.cancel-warning-btn').click(); await wait(100);
 assert(!document.querySelector('.warning-modal'),'cancel warning');
 assert(![...document.querySelectorAll('*')].some(e=>getComputedStyle(e).backdropFilter && getComputedStyle(e).backdropFilter!=='none'),'backdrop filter');
 return 'PASS: presets repeated open/apply, settings preview, warning updates/pause/cancel';
})().then(x=>window.webkit.messageHandlers.result.postMessage(x)).catch(e=>window.webkit.messageHandlers.result.postMessage('FAIL: '+e.message+' BODY: '+document.body.innerHTML.slice(0,1800)));
'''
def receive(manager,msg):
 global result
 text=msg.get_js_value().to_string(); print(text,flush=True); result=0 if text.startswith('PASS') else 1; Gtk.main_quit()
manager.register_script_message_handler('result'); manager.connect('script-message-received::result',receive)
def loaded(view,event):
 if event==WebKit2.LoadEvent.FINISHED:
  GLib.timeout_add(1200,lambda: (view.evaluate_javascript(checks,-1,None,None,None,None,None),False)[1])
view.connect('load-changed',loaded); view.load_uri('http://127.0.0.1:1420')
GLib.timeout_add_seconds(25,lambda: (print('TIMEOUT',flush=True),Gtk.main_quit(),False)[2]); Gtk.main(); raise SystemExit(result)
