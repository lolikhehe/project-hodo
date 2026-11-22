import{A as e,F as t,N as n,O as r,P as i,T as a,w as o}from"./vendor-lit-MQ4DzmMc.js";import{f as s,t as c}from"./index-NFRVgqV2.js";var l,u=(l=class extends a{render(){return r`
      <a href="/about/terms.php"
        >${i(t`Terms of Service (last updated ${this.termsLastUpdated})`)}</a
      >
    `}get termsLastUpdated(){return new Intl.DateTimeFormat(s(),{year:`numeric`,month:`numeric`,day:`numeric`}).format(new Date(2014,11,31))}},l.styles=e`
    :host {
      backdrop-filter: blur(2rem);
      -webkit-backdrop-filter: blur(2rem);
      font-weight: 200;
      font-size: 1.1rem;
      text-align: center;
      background-color: rgba(0, 0, 0, 0.7);
      padding: 0.5rem;
    }

    a:link,
    a:visited {
      color: white;
      text-decoration: none;
    }

    a:hover {
      text-decoration: underline;
    }
  `,l);u=c([o(`app-footer`),n()],u);export{u as AppFooter};