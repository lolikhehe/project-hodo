const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["offshoot_assets/metadata-link-RLXFkDb1.js","offshoot_assets/metadata-link-ClJ7nmQL.js","offshoot_assets/vendor-lit-MQ4DzmMc.js","offshoot_assets/external-link-snEq6XNi.js","offshoot_assets/search-link-BvHkWV2g.js","offshoot_assets/metadata-search-link-BjUeGzEk.js","offshoot_assets/source-link-ByTr5GRC.js","offshoot_assets/language-link-CedaT4ry.js","offshoot_assets/license-formatter-BVbhB1uQ.js","offshoot_assets/urn-link-BODKs9xn.js","offshoot_assets/unsafe-html-LYkx4y9w.js","offshoot_assets/custom-prefix-link-COiPrVng.js","offshoot_assets/sanitize-html-DtB1Wa_K.js"])))=>i.map(i=>d[i]);
import{A as e,C as t,E as n,I as r,N as i,O as a,P as o,S as s,T as c,a as l,n as u,s as d,v as f,w as p}from"./vendor-lit-MQ4DzmMc.js";import{a as m}from"./vendor-shared-62wh2Xkx.js";import"./vendor-topnav-Dcg-INpV.js";import{_ as h,h as g,t as _}from"./index-NFRVgqV2.js";import{i as v,l as y,n as b}from"./dist-naAOPxYm.js";import{n as x,t as S}from"./dist-C2wBh3A_.js";import{l as C}from"./dist-P4PnKJsG.js";import{t as w}from"./capitalize-BPJI7v6S.js";import{n as T}from"./truncate-screenname-igLoMw8l.js";function E(e){return/<br ?\/?>|<p>/i.test(e)?e:e.replace(/(\r?\n)/gi,`<br>`)}function D(e,t){return!e||e.length===0?!1:e.some(e=>{if(e===t||e===`/`)return!0;let n=e.endsWith(`/`)?e:`${e}/`;return t.startsWith(n)})}function ee(e){return D(e?.privs,`/`)}function O(e,t){if(t.mediatype?.value!==`texts`)return;let n=new URLSearchParams({b_id:e}),r=t.scanningcenter?.value,i=t.sponsor?.value,a=t.contributor?.value;return r&&n.set(`b_n`,r),i&&n.set(`b_p`,i),a&&n.set(`b_l`,a),n.set(`add`,`1`),`https://books-yaz.archive.org/biblio.php?${n}`}async function k(e,t,n){return{uploader:await A(e,t),biblio:O(t,n)}}async function A(e,t){try{return(await(await e.metadataService.get()).fetchMetadataValue(t,`metadata/uploader`)).success||void 0}catch{return}}var j=class extends c{constructor(...e){super(...e),this.expanded=!1}render(){let e=this.expanded?o(`Show Less`):o(`Show More`);return a`
      <button
        id="collapser-btn"
        aria-expanded=${this.expanded?`true`:`false`}
        @click=${this.buttonClicked}
      >
        ${e}${this.context?a`<ia-sr-only-text>&nbsp;${this.context}</ia-sr-only-text>`:n}
      </button>
    `}buttonClicked(){this.dispatchEvent(new CustomEvent(`collapserToggle`))}static get styles(){return e`
      #collapser-btn {
        min-height: 24px;
        margin: 10px 0;
        padding: 3px 12px;
        border: 1px solid #ccc;
        background: none;
        appearance: none;
        color: #767676;
        font-size: 10px;
        font-weight: 200;
        text-transform: uppercase;
        overflow-anchor: none;
        vertical-align: top;
        cursor: pointer;
      }
    `}};_([t({type:Boolean,reflect:!0})],j.prototype,`expanded`,void 0),_([t({type:String})],j.prototype,`context`,void 0),j=_([p(`collapser-button`),i()],j);function M(e){return new m({generator:e})}var N={metadataLink:()=>M(async()=>new(await(h(()=>g(()=>import(`./metadata-link-RLXFkDb1.js`),__vite__mapDeps([0,1,2,3]))))).MetadataLinkFormatter),searchLink:()=>M(async()=>new(await(h(()=>g(()=>import(`./search-link-BvHkWV2g.js`),__vite__mapDeps([4,2,5]))))).SearchLinkFormatter),sourceLink:()=>M(async()=>new(await(h(()=>g(()=>import(`./source-link-ByTr5GRC.js`),__vite__mapDeps([6,2,1,3,5]))))).SourceLinkFormatter),languageLink:()=>M(async()=>new(await(h(()=>g(()=>import(`./language-link-CedaT4ry.js`),__vite__mapDeps([7,2,5]))))).LanguageLinkFormatter),license:()=>M(async()=>new(await(h(()=>g(()=>import(`./license-formatter-BVbhB1uQ.js`),__vite__mapDeps([8,2]))))).LicenseFormatter),urnLink:()=>M(async()=>new(await(h(()=>g(()=>import(`./urn-link-BODKs9xn.js`),__vite__mapDeps([9,2,3,5]))))).UrnLinkFormatter),unsafeHtml:()=>M(async()=>new(await(h(()=>g(()=>import(`./unsafe-html-LYkx4y9w.js`),__vite__mapDeps([10,2]))))).UnsafeHtmlFormatter)};function P(e,t){return()=>M(async()=>new(await(h(()=>g(()=>import(`./custom-prefix-link-COiPrVng.js`),__vite__mapDeps([11,1,2,3]))))).CustomPrefixLinkFormatter(e,t))}function F(e){return()=>M(async()=>new(await(h(()=>g(()=>import(`./sanitize-html-DtB1Wa_K.js`),__vite__mapDeps([12,2]))))).SanitizeHTMLFormatter(e))}function I(){return{"access-restricted-item":o(`access restricted`),addeddate:o(`date added`),call_number:o(`call number`),camera:o(`camera`),catalog_number:o(`product code`),closed_captioning:o(`closed captioning`),collection:o(`collection`),collection_added:o(`collection added`),collection_set:o(`collection set`),color:o(`color`),contact:o(`contact information`),contributor:o(`contributor`),"copyright-evidence":o(`copyright evidence`),"copyright-evidence-date":o(`copyright evidence date`),"copyright-evidence-operator":o(`copyright evidence reviewed by`),"copyright-region":o(`copyright region`),coverage:o(`location`),creator:o(`creator`),"creator-alt-script":o(`creator (alternate script)`),creator_alt_script:o(`creator (alternate script)`),date:o(`publication date`),"external-identifier":o(`external identifier`),foldoutcount:o(`foldout count`),identifier:o(`identifier`),"identifier-ark":o(`ARK identifier`),"identifier-bib":o(`bibliographic identifier`),imagecount:o(`pages`),language:o(`language`),lcamid:o(`left camera ID`),licenseurl:o(`license`),mediatype:o(`media type`),"oclc-id":o(`WorldCat (source edition)`),ocr:o(`OCR`),ocr_converted:o(`OCR converted`),ocr_module_version:o(`OCR module version`),openlibrary_edition:o(`Open Library edition`),openlibrary_work:o(`Open Library work`),page_number_confidence:o(`page number confidence`),page_number_module_version:o(`page number module version`),"possible-copyright-status":o(`possible copyright status`),ppi:o(`PPI`),publicdate:o(`date made public`),publisher:o(`publisher`),rcamid:o(`right camera ID`),related_collection:o(`related collections`),"reviews-allowed":o(`reviews allowed`),rights:o(`rights`),"rights-holder":o(`rights holder`),rights_holder:o(`rights holder`),runtime:o(`run time`),scandate:o(`scan date`),scanner:o(`scanner`),scanningcenter:o(`scanning center`),sound:o(`sound`),sponsor:o(`sponsor`),subject:o(`subject`),taper:o(`taped by`),"title-alt-script":o(`title (alternate script)`),title_alt_script:o(`title (alternate script)`),topics:o(`topics`),transferer:o(`transferred by`),year:o(`year`)}}function L(e){return w(I()[e]??e)}var R=class{get title(){return this.titleOverride==null?this.key==null?``:L(this.key):this.titleOverride}async value(){let e=this.rawMetadataValues;return e.length===0?n:await this.render(e)}get shouldDisplay(){return this.rawMetadataValues.length>0}get rawMetadataValues(){if(this.key==null)return[];let e=this.rawMetadata[this.key];return e==null?[]:Array.isArray(e)?e:[e]}async render(e){let{key:t}=this,n=await this.formatter.get(),r=e.map(e=>n.format(e,t));return this.renderValues(r)}renderValues(e){if(e.length===0)return n;if(e.length===1)return e[0];switch(this.valueDisplayStyle){case`semicolon`:return a`${l(e,`; `)}`;case`comma`:return a`${l(e,`, `)}`;default:return a`<ul>
          ${e.map(e=>a`<li>${e}</li>`)}
        </ul>`}}get metadata(){return this.item?.metadata}get rawMetadata(){return this.metadata?.rawMetadata??{}}setup(e,t,n){this.item=e,this.key=t,n?.title!=null&&(this.titleOverride=n.title)}constructor(e,t,n){this.valueClass=void 0,this.valueDisplayStyle=`ul`,this.formatter=N.metadataLink(),e!=null&&(this.item=e),t!=null&&(this.key=t),n?.formatter!=null&&(this.formatter=n.formatter),n?.valueClass!=null&&(this.valueClass=n.valueClass),n?.title!=null&&(this.titleOverride=n.title)}};_([y()],R.prototype,`rawMetadataValues`,null);var z=class extends R{constructor(e,t,n){super(e,t,{...n,formatter:N.searchLink()}),this.fieldParser=new x(S.shared),this.valueDisplayStyle=`comma`}async value(){let e=this.rawMetadataValues;if(e.length===0)return n;let t=e.flatMap(e=>this.fieldParser.parseValue(e));return await this.render(t)}};function B(e,t){return()=>new R(void 0,void 0,{...t,formatter:e()})}var V=class extends R{constructor(...e){super(...e),this.titleOverride=``}get shouldDisplay(){return!1}async value(){return n}},H=class extends R{get shouldDisplay(){let e=this.metadata?.segments?.value;return!(e?.includes(`Title #`)&&e.includes(`a href=`))&&super.shouldDisplay}},U={hidden:()=>new V,segments:()=>new H,delimitedSearchLinks:()=>new z,metadataLink:B(N.metadataLink),searchLink:B(N.searchLink),sourceLink:B(N.sourceLink),languageLink:B(N.languageLink),urnLink:B(N.urnLink),unsafeHTML:B(N.unsafeHtml),sanitizingHTML:B(F(T),{valueClass:`sanitized-html`}),collectionLink:B(P(`/details/`,{tab:`about`})),oclcLink:B(P(`https://www.worldcat.org/oclc/`)),olBookLink:B(P(`https://openlibrary.org/books/`)),olWorkLink:B(P(`https://openlibrary.org/works/`))},W=`adder.admincmd.admincont.admintime.adminuser.backup_location.collection_layout.contacted_address.contacted_by.curatenote.curation.filesxml.foldout-operator.hidden.homepage.identifier-access.md5contents.md5s.noarchivetorrent.nre.operator.origin-contact.origin-note.origin-organization.origin-physical.paginated.pick.prevtask.public.repub_seconds.repub_state.republisher.scanfee.shiptracking.show_search_by_year.sponsor.sponsordate.update.updatedate.updater.loans__status__last_loan_date.loans__status__last_waitlist_date.loans__status__num_history.loans__status__num_loans.loans__status__num_waitlist.loans__status__status.analytics__field_names.analytics__field_count.analytics__doc_size_md.analytics__doc_size_se.analytics__rejected_fields.analytics__raw_value_sources.analytics__maker.analytics__maker_hostname.analytics__maker_worker.analytics__maker_repository.analytics__maker_branch.analytics__maker_commit.analytics__issues.analytics__maketime.analytics__made_for_es_version.analytics__made_for_index.analytics__debug_switches.analytics__debug_mdrecord.analytics__metadata_validation.analytics__nonconforming_fields.analytics__parsed_date_service_succeeded.analytics__features.analytics__features__any.analytics__features__flagged.analytics__features__collection.analytics__features__user.analytics__features__data.analytics__features__open`.split(`.`).reduce((e,t)=>(e[t]=U.hidden,e),{}),G=e`
  .sanitized-html img {
    max-width: 100%;
    height: auto;
  }
`,K=`data:image/svg+xml,%3csvg%20viewBox='0%200%20100%20100'%20xmlns='http://www.w3.org/2000/svg'%3e%3cpath%20d='m100%207.78013601c0-2.14552613-.7593357-3.978597-2.278007-5.4992126-1.5186713-1.52061561-3.3493984-2.28092341-5.4921813-2.28092341h-84.54977287c-2.08268321%200-3.88336049.7603078-5.40203183%202.28092341-1.51867133%201.5206156-2.278007%203.35368647-2.278007%205.4992126v51.49262709c0%202.0853495.75933567%203.8883321%202.278007%205.4089477%201.51867134%201.5206156%203.31934862%202.2809234%205.40203183%202.2809234h10.53361537l.3571304%2033.0373658%2032.4087237-33.0373658h41.2468361c2.1427829%200%203.97351-.7603078%205.4921813-2.2809234s2.278007-3.3235982%202.278007-5.4089477z'/%3e%3c/svg%3e`,q=e`
  :host {
    font-size: 1.6rem;
  }

  a:link {
    color: var(--ia-theme-link-color, #4b64ff);
    text-decoration: none;
  }

  a:link:hover {
    text-decoration: underline;
  }

  dl {
    display: grid;
    grid-template-columns: 1fr 2fr;
    margin: 0 0 0;
  }

  dl > dt,
  dl > dd {
    word-break: break-word;
  }

  dl > dd {
    font-weight: 200;
  }

  dl > dd ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  dl:not(.expanded) .collapsible {
    display: none;
  }

  h2 {
    font-size: 18px;
    line-height: 1.1;
    margin-top: 20px;
    margin-bottom: 10px;
    font-weight: 500;
  }

  .metadata-urn-row {
    display: flex;
  }

  .metadata-urn-link,
  .metadata-external-icon {
    margin: 1px 0 -1px 5px;
  }

  .metadata-urn-icon,
  .metadata-external-icon {
    width: 13px;
    height: 13px;
  }

  .license-icons img {
    width: 20px;
    margin-left: 5px;
    vertical-align: middle;
  }

  .license-icons img:first-child {
    margin-left: 0;
  }

  .metadata-description,
  .metadata-notes,
  .metadata-credits {
    font-size: 14px;
  }

  .metadata-description {
    margin: 20px 25px 20px 0;
  }

  .metadata-notes,
  .metadata-credits {
    margin: 0 25px 20px 0;
  }

  .status-message {
    margin: 20px 25px 20px 0;
    padding: 9px;
    font-size: 14px;
    background: #f5f5f5;
    border: 1px solid #e3e3e3;
    border-radius: 4px;
  }

  .metadata-shotlist {
    margin: 0 25px 20px 0;
  }

  .metadata-shotlist pre {
    display: block;
    box-sizing: border-box;
    padding: 9.5px;
    margin: 0 0 10px;
    font-family: var(
      --ia-theme-monospace-font-family,
      ui-monospace,
      SFMono-Regular,
      Menlo,
      Monaco,
      Consolas,
      'Courier New',
      monospace
    );
    font-size: 13px;
    line-height: 1.42857;
    color: #333;
    white-space: pre;
    word-break: break-all;
    word-wrap: break-word;
    background: #f5f5f5;
    border: 1px solid #cccccc;
    border-radius: 4px;
    overflow: auto;
  }

  .etree-link {
    font-size: 12px;
  }
`,J=()=>new R,Y=class{constructor(e,t,n=J){this.item=e,this.handlers=t,this.defaultHandler=n}handlerFor(e){let t=(this.handlers[e]||this.defaultHandler)();return t.setup(this.item,e),t}allHandlers(){let e=Object.keys(this.item?.metadata?.rawMetadata??{}),t={};for(let n of e)t[n]=this.handlerFor(n);return t}};_([y()],Y.prototype,`allHandlers`,null);var X=class{constructor(e){this.itemDetails=e}get metadata(){return this.itemDetails.public_metadata}get mediatype(){return this.metadata?.mediatype?.value}get collection(){return this.collections[0]}get collections(){return this.metadata?.collection?.values??[]}get isEtree(){return this.mediatype===`etree`||this.collections.includes(`etree`)}get isGratefulDead(){return this.collections.includes(`GratefulDead`)}get isTvArchive(){return(this.collections.includes(`tvarchive`)&&this.mediatype!==`data`)??!1}get isTexts(){return this.mediatype===`texts`}get isMovie(){return this.mediatype===`movies`}get isEducation(){let e=this.metadata?.mediatype?.rawValue;return e==null?!1:new v(e).value===`education`}get isSamplesOnly(){return this.collections.includes(`samples_only`)}get isStreamOnly(){return this.collections.includes(`stream_only`)}},Z=class e{get item_size(){return this.rawData.item_size}get files_count(){return this.rawData.files_count}get month(){return this.rawData.month}get week(){return this.rawData.week}get downloads(){return this.rawData.downloads}get num_favorites(){return this.rawData.num_favorites}get title_message(){return this.rawData.title_message}get primary_collection(){return this.rawData.primary_collection}get thumbnail_url(){return this.rawData.thumbnail_url}get num_reviews(){return this.rawData.num_reviews}get uploader_details(){return this.rawData.uploader_details}get public_metadata(){let e=this.rawData.public_metadata;if(e){if(e instanceof b)return e;if(typeof e==`object`)return new b(e)}}get part_of(){let e=this.rawData.part_of;if(e)return Array.isArray(e)?e:[e]}get visible_reviews(){return this.reviews_metadata.filter(e=>e.reviewer_account_status!==`locked`)}get reviews_metadata(){return(this.rawData.reviews_metadata??[]).map(e=>e instanceof C?e:new C(e))}get speech_vs_music_asr_metadata(){return this.rawData.speech_vs_music_asr_metadata}constructor(e){this.rawData=e}static fromPPSExtraInfo(t){return new e(t.rawResponse)}};_([y()],Z.prototype,`public_metadata`,null),_([y()],Z.prototype,`visible_reviews`,null),_([y()],Z.prototype,`reviews_metadata`,null);var Q=[`keywords_`,`zoterokey`],$=class extends c{constructor(...e){super(...e),this.handlers={},this.numUnhiddenRows=12,this.allRowsExpanded=!1,this.sortedVisibleMetadata=[],this.onLocaleStatus=e=>{let{status:t}=e.detail;t===`ready`&&this.setupVisibleMetadata()}}render(){return this.sortedVisibleMetadata.length===0?n:a`
      <dl
        id="metadata-list"
        class=${f(this.allRowsExpanded?`expanded`:void 0)}
      >
        ${d(this.sortedVisibleMetadata,({handler:e},t)=>{let n=t>=this.numUnhiddenRows?`collapsible`:void 0,r=[n,e.valueClass].filter(e=>e!=null),i=r.length>0?r.join(` `):void 0;return a`
            <dt class=${f(n)}>${e.title}</dt>
            <dd class=${f(i)}>${u(e.value())}</dd>
          `})}
      </dl>
      ${this.collapserBtnTemplate}
    `}connectedCallback(){super.connectedCallback(),window.addEventListener(r,this.onLocaleStatus)}disconnectedCallback(){window.removeEventListener(r,this.onLocaleStatus),super.disconnectedCallback()}willUpdate(e){(e.has(`metadata`)||e.has(`handlers`))&&this.setupVisibleMetadata()}async setupVisibleMetadata(){if(this.metadata==null){this.sortedVisibleMetadata=[];return}let e=new Y(new X(new Z({public_metadata:this.metadata})),this.handlers,this.defaultHandler??(()=>new R)).allHandlers();this.sortedVisibleMetadata=Object.entries(e).map(([e,t])=>({key:e,handler:t})).filter(({key:e})=>!Q.some(t=>e.startsWith(t))).filter(({handler:e})=>e.shouldDisplay).sort((e,t)=>e.handler.title.localeCompare(t.handler.title))}get collapserBtnTemplate(){return!this.sortedVisibleMetadata||this.sortedVisibleMetadata.length<=this.numUnhiddenRows?n:a`
      <collapser-button
        ?expanded=${this.allRowsExpanded}
        .context=${this.collapserContext}
        @collapserToggle=${this.collapserToggled}
      ></collapser-button>
    `}collapserToggled(){this.allRowsExpanded=!this.allRowsExpanded,this.dispatchEvent(new CustomEvent(`metadataCollapseStateChange`,{detail:{expanded:this.allRowsExpanded}}))}static get styles(){return[q,G]}};_([t({type:Object})],$.prototype,`metadata`,void 0),_([t({type:Object})],$.prototype,`handlers`,void 0),_([t({type:Object})],$.prototype,`defaultHandler`,void 0),_([t({type:Number})],$.prototype,`numUnhiddenRows`,void 0),_([t({type:String})],$.prototype,`collapserContext`,void 0),_([s()],$.prototype,`allRowsExpanded`,void 0),_([s()],$.prototype,`sortedVisibleMetadata`,void 0),$=_([p(`collapsible-metadata-list`),i()],$);export{E as _,G as a,V as c,P as d,N as f,D as g,ee as h,K as i,z as l,k as m,X as n,W as o,F as p,q as r,U as s,Z as t,R as u};