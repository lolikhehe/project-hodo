import{E as e,O as t,m as n}from"./vendor-lit-MQ4DzmMc.js";import{t as r}from"./external-link-snEq6XNi.js";function i(e){try{return[`http:`,`https:`].includes(new URL(e).protocol)}catch{return!1}}function a(e){try{return/^\/([^\/]|$)/.test(e)&&!!new URL(e,window.location.origin)}catch{return!1}}function o(o,s=o,{icon:c=!1}={}){let l=a(o),u=i(o)||l,d=!l&&!/^https?:\/\/([a-z0-9-]+\.)*archive\.org(\/|$)/i.test(o),f=d?`external nofollow noreferrer noorigin`:e,p=n({"metadata-link":!0,external:d}),m=t`<img
    class="metadata-external-icon"
    src=${r}
    alt="external link"
    loading="lazy"
  />`;return u?t`<a class=${p} rel=${f} href=${o}>
        ${s}${c?m:e}
      </a>`:t`${o}`}export{i as n,o as t};