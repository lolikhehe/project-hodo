import{n as e}from"./rolldown-runtime-QTnfLwEv.js";import{O as t}from"./vendor-lit-MQ4DzmMc.js";import{i as n}from"./dist-BqHmxVdQ.js";import"./login-page-BiUY-YWQ.js";var r=e({showLoginModal:()=>i});function i(e,r,i){e.setAttribute(`id`,`login-modal`);let a=()=>{e.removeAttribute(`id`)},o=async()=>{a(),e.closeModal(),await i()},s=new n;s.showHeaderLogo=!1,e.showModal({config:s,customModalContent:t`
      <login-page
        style="--loginPageContainerMinHeight: auto"
        .appServices=${r}
        .successCallback=${o}
      ></login-page>
    `,userClosedModalCallback:a})}export{r as n,i as t};