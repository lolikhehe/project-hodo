const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["offshoot_assets/create-description-editor-CkLBnKZw.js","offshoot_assets/is-allowed-link-url-Ff6CmaCA.js"])))=>i.map(i=>d[i]);
import{A as e,C as t,E as n,F as r,O as i,P as a,S as o,T as s,k as c,m as l,o as u,p as d,v as f,w as p,x as m}from"./vendor-lit-MQ4DzmMc.js";import{_ as h,h as g,t as _}from"./index-NFRVgqV2.js";import"./ia-status-indicator-Bd61y7bd.js";import{t as v}from"./upload-service-request-error-DxHEPIQ1.js";import{n as y,r as ee,t as b}from"./is-allowed-link-url-Ff6CmaCA.js";var te=i`<svg
  class="working-glyph"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  <path d="M5 2.5h14M5 21.5h14" />
  <path
    d="M7 2.5c0 4 2.4 6.3 5 9.5-2.6 3.2-5 5.5-5 9.5M17 2.5c0 4-2.4 6.3-5 9.5 2.6 3.2 5 5.5 5 9.5"
  />
  <path d="M9 19.5h6L12 16Z" fill="currentColor" stroke-width="1" />
</svg>`;function x(e){return a(e===1?`1 file`:r`${e.toLocaleString()} files`)}function ne(e){return a(e===1?`1 folder`:r`${e.toLocaleString()} folders`)}var S=1e3,re=[e=>a(r`${e} KB`),e=>a(r`${e} MB`),e=>a(r`${e} GB`),e=>a(r`${e} TB`),e=>a(r`${e} PB`)];function C(e){if(!Number.isFinite(e)||e<S){let t=Math.max(0,Math.floor(e))||0;return a(t===1?`1 byte`:r`${t.toLocaleString()} bytes`)}let t=0,n=e/S;for(;t<re.length-1&&ie(n,t)>=S;)n/=S,t+=1;let i=ie(n,t),o=t>0&&i<10,s=i.toLocaleString(void 0,{minimumFractionDigits:+!!o,maximumFractionDigits:+!!o});return re[t](s)}function ie(e,t){let n=Math.round(e*10)/10;return t>0&&n<10?n:Math.round(e)}function ae(e){let t=e.added.length,n=e.skipped.filter(e=>e.reason!==`system`).length;return[a(t===0?`No files added.`:t===1?`1 file added.`:r`${t.toLocaleString()} files added.`),n===0?void 0:a(r`${n.toLocaleString()} skipped.`),le(e.list.totals)].filter(e=>e!==void 0).join(` `)}function oe(e){let t=e.removed.length;return`${a(t===1?`1 file removed.`:r`${t.toLocaleString()} files removed.`)} ${le(e.list.totals)}`}function se(e){let t=e.filter(e=>e.reason===`duplicate`),n=e.filter(e=>e.reason===`empty`),i=e.filter(e=>e.reason===`invalidName`),o=t.length>3?[a(r`${t.length.toLocaleString()} files were skipped: files with those names are already in the list.`)]:t.map(ce),s=n.length>3?[a(r`${n.length.toLocaleString()} files are empty (0 bytes), so they were skipped.`)]:n.map(e=>{let t=e.path;return a(r`${t} is empty (0 bytes), so it was skipped.`)}),c=i.length>3?[a(r`${i.length.toLocaleString()} files were skipped: their names have characters or a length the uploader can't send.`)]:i.map(e=>{let t=e.path;return a(r`${t} was skipped: its name has characters or a length the uploader can't send.`)});return[...o,...s,...c]}function ce(e){let t=e.path;return e.isFolder?a(r`${t} was skipped: a folder with that name is already in the list.`):a(r`${t} was skipped: a file with that name is already in the list.`)}function le(e){return e.fileCount===0?a(`No files yet.`):a(r`${x(e.fileCount)}, ${C(e.totalBytes)}.`)}function ue(e,t){let n=de(``,``,``);for(let r of e){let e=r.path.split(`/`),i=e.pop()??r.path,a=n;for(let n of e)a=fe(a,n,t),a.fileCount+=1,a.totalBytes+=r.file.size;a.children.push({kind:`file`,uploadFile:r,name:i})}return pe(n.children,0,0)}function de(e,t,n){return{id:e,name:t,path:n,children:[],subfolders:new Map,fileCount:0,totalBytes:0}}function fe(e,t,n){let r=e.subfolders.get(t);if(r)return r;let i=e.path?`${e.path}/${t}`:t,a=de(n.get(i)??`folder:${i}`,t,i);return e.subfolders.set(t,a),e.children.push({kind:`folder`,folder:a}),a}function pe(e,t,n){let r={depth:t,displayDepth:Math.min(t,5),level:n+1,indentCapped:t>5};return e.map(e=>e.kind===`file`?me(e.uploadFile,e.name,r):he(e.folder,r))}function me(e,t,n){return{kind:`file`,id:e.id,uploadFile:e,name:t,size:e.file.size,...n}}function he(e,t){let n=ge(e),r=n[n.length-1]??e,i=t.level-1+n.length;return{kind:`folder`,id:e.id,folderIds:n.map(e=>e.id),names:n.map(e=>e.name),path:e.path,nestedFolderCount:n.length,fileCount:e.fileCount,totalBytes:e.totalBytes,...t,children:pe(r.children,t.depth+1,i)}}function ge(e){let t=[e],n=e;for(;;){let e=n.children.length===1?n.children[0]:void 0;if(e?.kind!==`folder`)return t;n=e.folder,t.push(n)}}function _e(e){let t=e.split(`/`);return t.pop(),t.map((e,n)=>t.slice(0,n+1).join(`/`))}var ve=1900,ye=/[\x00-\x1f\x7f]/;function be(e){if(ye.test(e))return!1;try{return encodeURIComponent(`/${e}`).length<=ve}catch{return!1}}var xe=new Set([`.ds_store`,`thumbs.db`,`desktop.ini`]);function Se(e){let t=e.slice(e.lastIndexOf(`/`)+1);return xe.has(t.toLowerCase())}var Ce=class e{static empty(){return new e({files:[],folderIds:new Map,systemSkippedPaths:[]})}constructor(e){this.state=e}get files(){return this.state.files}get tree(){return this.treeCache??(this.treeCache=ue(this.state.files,this.state.folderIds)),this.treeCache}get totals(){return this.totalsCache??(this.totalsCache=this.computeTotals()),this.totalsCache}add(t){let n=new Set(this.state.files.map(e=>e.path)),r=new Map(this.state.folderIds),i=this.topFoldersAlreadyInList(t),a=new Set,o=[],s=[],c=[];for(let e of t){let t=Ee(e),l=De(t);if(l!==void 0&&i.has(l)){a.has(l)||(a.add(l),s.push({path:l,reason:`duplicate`,isFolder:!0}));continue}if(Se(t)){s.push({path:t,reason:`system`}),c.push(t);continue}if(!be(t)){s.push({path:t,reason:`invalidName`});continue}if(e.file.size===0){s.push({path:t,reason:`empty`});continue}let u=_e(t);if(n.has(t)||r.has(t)||u.some(e=>n.has(e))){s.push({path:t,reason:`duplicate`});continue}let d={id:Te(`file`),file:e.file,path:t};o.push(d),n.add(t);for(let e of u)r.has(e)||r.set(e,Te(`folder`))}return{list:o.length===0&&c.length===0?this:new e({files:[...this.state.files,...o],folderIds:r,systemSkippedPaths:[...this.state.systemSkippedPaths,...c]}),added:o,skipped:s}}remove(t){let n=this.folderPathForId(t),r=n===void 0?e=>e.id===t:e=>e.path.startsWith(`${n}/`),i=this.state.files.filter(r);if(i.length===0)return{list:this,removed:i};let a=this.state.files.filter(e=>!r(e)),o=new Set(a.flatMap(e=>_e(e.path)));return{list:new e({files:a,folderIds:new Map([...this.state.folderIds].filter(([e])=>o.has(e))),systemSkippedPaths:a.length===0?[]:this.state.systemSkippedPaths.filter(e=>n===void 0||!e.startsWith(`${n}/`))}),removed:i}}topFoldersAlreadyInList(e){let t=new Set(this.state.files.map(e=>e.path).filter(e=>!e.includes(`/`))),n=new Set;for(let r of e){let e=De(Ee(r));e!==void 0&&(this.state.folderIds.has(e)||t.has(e))&&n.add(e)}return n}folderPathForId(e){for(let[t,n]of this.state.folderIds)if(n===e)return t}computeTotals(){let e=0,t;for(let n of this.state.files){let{size:r}=n.file;e+=r,(!t||r>t.file.size)&&(t=n)}return{fileCount:this.state.files.length,folderCount:this.state.folderIds.size,totalBytes:e,largestFile:t,systemFilesSkipped:this.state.systemSkippedPaths.length}}},we=0;function Te(e){return we+=1,`${e}-${we}`}function Ee(e){return e.path.replace(/^\/+/,``)||e.file.name}function De(e){let t=e.indexOf(`/`);return t===-1?void 0:e.slice(0,t)}var Oe=`Internet Archive Uploader 2.0.0`,w=[`title`,`identifier`,`description`,`subjects`,`collection`],ke=`test_collection`;function Ae(e){return e.split(`,`).map(e=>e.trim()).filter(e=>e!==``)}function je(e,t={}){let n=[],r=new Map,i=e=>{let t=(r.get(e)??0)+1;return r.set(e,t),`x-archive-meta${String(t).padStart(2,`0`)}-${e}`},a=(e,t)=>{n.push([i(e),Me(t)])},o=(e,t)=>{let n=t.trim();n!==``&&a(e,n)};for(let t of Ae(e.subjects))a(`subject`,t);let s=e.collection.split(`:`);if(s.length>1){let[e,r]=s,i=e||t.inferredMediatype||``;n.push([`x-archive-meta-mediatype`,Me(i)]),a(`collection`,r)}let c=ee(e.description);c!==``&&a(`description`,c),o(`title`,e.title),o(`creator`,e.creator),o(`date`,e.date),o(`language`,e.language),e.testItem&&n.push([i(`collection`),ke]),e.licenseUrl!==``&&a(`licenseurl`,e.licenseUrl);for(let t of e.custom)t.key!==``&&t.value.trim()!==``&&a(t.key,t.value);return a(`scanner`,Oe),n}function Me(e){return`uri(${encodeURIComponent(e)})`}var Ne=5;function Pe(e){return Math.max(e,Ne)}var Fe=1,Ie=3,Le=class{constructor(e){this.currentStatus={state:`idle`},this.value=``,this.userOwnsValue=!1,this.generation=0,this.check=e.check,this.onChange=e.onChange,this.debounceMs=e.debounceMs??500;let t=e.maxIdLength??0,n=t>0;this.maxLength=n?Math.min(t,80):80}get status(){return this.currentStatus}async propose(e){if(this.userOwnsValue)return;let t=this.startNewCall(),n=e.trim();if(n===``){this.value=``,this.setStatus({state:`idle`});return}this.setStatus({state:`checking`,identifier:n});let r=await this.runCheck(n,t,Fe);if(!r)return;let i=r.legal&&this.fits(n),a=i&&r.available?n:this.usableSuggestions(r)[0];if(!a&&r.check_failed){this.setStatus({state:`error`,identifier:n});return}if(!a){this.value=``,this.setStatus({state:i?`taken`:`illegal`,identifier:n,suggestions:[]});return}return this.value=a,this.setStatus({state:`available`,identifier:a}),a}edit(e,t={}){if(e===this.value&&this.currentStatus.state!==`error`)return;let n=this.startNewCall();if(this.value=e,this.userOwnsValue=e!==``,e===``){this.setStatus({state:`idle`});return}if(!this.fits(e)){this.setStatus({state:`illegal`,identifier:e,suggestions:[]});return}if(this.setStatus({state:`checking`,identifier:e}),t.immediate){this.checkEdit(e,n);return}this.debounceTimer=setTimeout(()=>{this.debounceTimer=void 0,this.checkEdit(e,n)},this.debounceMs)}markTaken(e,t){this.startNewCall(),this.value=e,this.setStatus({state:`taken`,identifier:e,suggestions:t.filter(e=>this.fits(e))})}dispose(){this.startNewCall()}async checkEdit(e,t){let n=await this.runCheck(e,t,Ie);if(!n)return;let r=n.legal&&this.fits(e);if(r&&n.available){this.setStatus({state:`available`,identifier:e});return}let i=this.usableSuggestions(n);if(r&&n.check_failed===!0&&i.length===0){this.setStatus({state:`error`,identifier:e});return}this.setStatus({state:r?`taken`:`illegal`,identifier:e,suggestions:i})}async runCheck(e,t,n){let r=Pe(this.maxLength);try{let i=await this.check(e,{maxLength:r,maxSuggestions:n});return t===this.generation?i:void 0}catch{t===this.generation&&this.setStatus({state:`error`,identifier:e});return}}startNewCall(){return clearTimeout(this.debounceTimer),this.debounceTimer=void 0,this.generation+=1,this.generation}usableSuggestions(e){return e.suggestions.filter(e=>this.fits(e))}fits(e){return e.length<=this.maxLength}setStatus(e){this.currentStatus=e,this.onChange(e)}},Re=new Set([`application/pdf`,`application/epub+zip`,`application/postscript`,`application/msword`,`application/vnd.oasis.opendocument.text`,`application/vnd.oasis.opendocument.presentation`,`application/vnd.ms-powerpoint`,`application/vnd.openxmlformats-officedocument.presentationml.presentation`,`application/vnd.openxmlformats-officedocument.wordprocessingml.document`]);function ze(e){let{name:t,type:n}=e;if(Re.has(n))return`texts`;if(n===`application/ogg`||n===``&&/\.(cdr|mov)$/.test(t))return`movies`;if(n.startsWith(`text`))return`texts`;if(n.startsWith(`audio`))return`audio`;if(n.startsWith(`video`))return`movies`;if(n.startsWith(`image`))return`image`;if(/_images\.(zip|tar)$/.test(t))return`texts`}var T={cc0:`https://creativecommons.org/publicdomain/zero/1.0/`,by:`https://creativecommons.org/licenses/by/4.0/`,bySa:`https://creativecommons.org/licenses/by-sa/4.0/`,byNd:`https://creativecommons.org/licenses/by-nd/4.0/`,byNc:`https://creativecommons.org/licenses/by-nc/4.0/`,byNcSa:`https://creativecommons.org/licenses/by-nc-sa/4.0/`,byNcNd:`https://creativecommons.org/licenses/by-nc-nd/4.0/`,publicDomainMark:`https://creativecommons.org/publicdomain/mark/1.0/`};function Be(){return[{value:``,label:a(`No license selected`)},{value:T.cc0,label:a(`CC0 1.0 (public domain dedication, no rights reserved)`)},{value:T.by,label:a(`CC BY 4.0 (attribution)`)},{value:T.bySa,label:a(`CC BY-SA 4.0 (attribution, share-alike)`)},{value:T.byNd,label:a(`CC BY-ND 4.0 (attribution, no derivatives)`)},{value:T.byNc,label:a(`CC BY-NC 4.0 (attribution, non-commercial)`)},{value:T.byNcSa,label:a(`CC BY-NC-SA 4.0 (attribution, non-commercial, share-alike)`)},{value:T.byNcNd,label:a(`CC BY-NC-ND 4.0 (attribution, non-commercial, no derivatives)`)},{value:T.publicDomainMark,label:a(`Public Domain Mark 1.0 (no known copyright)`)}]}var Ve=/^https?:\/\/(?:www\.)?creativecommons\.org\/(?:publicdomain\/(zero|mark)|licenses\/([^/?#]+))(?:[/?#].*)?$/;function He(e){let t=Ve.exec(e.trim().toLowerCase());if(!t)return;let[,n,r]=t;if(n===`zero`)return T.cc0;if(n===`mark`)return T.publicDomainMark;if(!r)return;let[i,...a]=r.split(`-`),o=[`nc`,`nd`,`sa`];if(!(i===`by`&&a.every(e=>o.includes(e))))return;let s=!a.includes(`nd`),c=a.includes(`nc`),l=a.includes(`sa`);return s?c?l?T.byNcSa:T.byNc:l?T.bySa:T.by:c?T.byNcNd:T.byNd}var Ue=new Set([`nobrowsercheck`,`title`,`identifier`,`description`,`subject`,`creator`,`date`,`collection`,`test_item`,`language`,`licenseurl`,`max_id_length`,`suggested_identifier`]),We=/^\d{4}(?:-\d{2}(?:-\d{2})?)?$/;function Ge(e){let t=Ke(e),n={},r=[],i=(e,t)=>{t.trim()!==``&&r.push({key:e,value:t})},a=e=>{let[n,...r]=t.get(e)??[];for(let t of r)i(e,t);return n},o=a(`description`),s=qe(t.get(`subject`)),c=a(`title`),l=a(`identifier`),u=a(`collection`),d=a(`creator`),f=a(`date`),p=a(`test_item`),m=a(`language`),h=a(`licenseurl`),g=a(`suggested_identifier`);o!==void 0&&(n.description=o),s!==void 0&&(n.subjects=s),c!==void 0&&(n.title=c),d!==void 0&&(n.creator=d),p!==void 0&&(n.testItem=Number(p)===1);let _=f?.trim()??``;We.test(_)?n.date=_:i(`date`,_);let v=He(h??``);v&&(n.licenseUrl=v);let[y=``,...ee]=Je(m);y!==``&&(n.language=y);for(let e of ee)i(`language`,e);let[b=``,...te]=Je(u);for(let e of te)i(`collection`,e);for(let[e,n]of t)if(!Ue.has(e))for(let t of n)i(e,t);let x=l||g||``;x!==``&&!c&&(n.title=x);let ne=Ye(t.get(`max_id_length`)),S={metadata:n,extraRows:r};return b!==``&&(S.presetCollection=b),x!==``&&(S.suggestedIdentifier=x),l&&(S.explicitIdentifier=l),g&&(S.presetSuggestedIdentifier=g),ne&&(S.maxIdLength=ne),S}function Ke(e){let t=new Map,n=new Set;for(let[r,i]of e){let e=r.indexOf(`[`),a=e!==-1,o=a?r.slice(0,e):r;if(o===``)continue;if(!a){t.set(o,[i]),n.delete(o);continue}let s=n.has(o)?t.get(o):void 0;s?s.push(i):(t.set(o,[i]),n.add(o))}return t}function qe(e){if(e!==void 0)return e.map(e=>e.trim()).filter(e=>e!==``).join(`, `)}function Je(e){return e===void 0?[]:e.trim().split(/,\s*/)}function Ye(e){let[t]=e??[],n=t?.trim()??``,r=Number(n);if(!(n!==``&&Number.isFinite(r)))return;let i=Math.trunc(r);return i>0?i:void 0}var Xe=`/upload`;function Ze(e){let t=e.replace(/^\?/,``);return t===``?Xe:`${Xe}?${t}`}function Qe(e){return e.dataTransfer?.types.includes(`Files`)??!1}function $e(e,t){let n=t=>{switch(e.stage){case`idle`:case`identifierTaken`:return;case`checkingIdentifier`:return{state:`waiting`};case`uploading`:{if(t<e.fileIndex)return{state:`done`};if(t>e.fileIndex)return{state:`waiting`};let{fileBytesSent:n,fileBytesTotal:r}=e;return{state:`uploading`,fraction:r>0?Math.min(n/r,1):0}}case`transferFailed`:return e.failedDuring===`identifierCheck`?{state:`waiting`}:t<e.fileIndex?{state:`done`}:t>e.fileIndex?{state:`waiting`}:{state:`failed`};case`processing`:case`taskFailed`:case`ready`:case`statusUnavailable`:return{state:`done`}}},r={};return t.forEach((e,t)=>{let i=n(t);i&&(r[e.id]=i)}),r}var et=/^[a-z][a-z\d_-]*$/,tt=[`identifier`,`title`,`description`,`mediatype`,`test_item`,`licenseurl`,`dir`,`prevtask`,`next_cmd`,`tester`,`key`,`search`,`currentFieldNum`,`multi`,`scribe`,`nre`,`admincmd`,`adminuser`,`admintime`,`admincont`,`stub_files`];function nt(e){let t={};return e.key===``?t.key=`missing`:et.test(e.key)?tt.includes(e.key)&&(t.key=`blocked`):t.key=`invalid`,e.value.trim()===``&&(t.value=`missing`),t.key!==void 0||t.value!==void 0?t:void 0}function rt(e){let t={};for(let n of e){let e=nt(n);e&&(t[n.id]=e)}return t}function it(e,t){switch(e){case`missing`:return a(`Enter a key.`);case`invalid`:return a(`Start with a lowercase letter, then use only lowercase letters, digits, hyphens and underscores.`);case`blocked`:return a(r`The key "${t}" is reserved and can't be used here.`)}}function at(){return a(`Enter a value.`)}function ot(e){return e.trim()===``?!1:(new DOMParser().parseFromString(e,`text/html`).body.textContent??``).trim()!==``}function st(e){let{metadata:t,hasFiles:n,identifierStatus:r}=e,i=t.collection.split(`:`)[1]??``,a=r.state===`available`&&t.identifier!==``&&r.identifier===t.identifier,o={title:t.title.trim()!==``,identifier:a,description:ot(t.description),subjects:Ae(t.subjects).length>0,collection:i.trim()!==``},s=e=>n?o[e]?`satisfied`:`missing`:`neutral`;return{states:{title:s(`title`),identifier:s(`identifier`),description:s(`description`),subjects:s(`subjects`),collection:s(`collection`)},missing:w.filter(e=>!o[e])}}function ct(e,t=new Date){let n=e.trim();if(n===``)return;let r=/^(\d{4})(?:-(\d{2})(?:-(\d{2}))?)?$/.exec(n);if(!r)return`format`;let[,i,a=`01`,o=`01`]=r,s=Number(i),c=Number(a),l=Number(o);if(c<1||c>12)return`month`;if(l<1||l>31)return`day`;if(l>ut(s,c))return`dayInMonth`;let u=new Date(t);if(u.setFullYear(s,c-1,l),u.setHours(0,0,0,0),u>t)return`future`}function lt(e){switch(e){case`format`:return a(`Use YYYY, YYYY-MM or YYYY-MM-DD, e.g. 1987 or 1987-06-21.`);case`month`:return a(`The month must be from 01 to 12.`);case`day`:return a(`The day must be from 01 to 31.`);case`dayInMonth`:return a(`That day doesn't exist in that month.`);case`future`:return a(`The date can't be in the future.`)}}function ut(e,t){return[31,e%4==0&&e%100!=0||e%400==0?29:28,31,30,31,30,31,31,30,31,30,31][t-1]}function dt(e){let{metadata:t,hasFiles:n,identifierStatus:r,now:i}=e,{states:a,missing:o}=st({metadata:t,hasFiles:n,identifierStatus:r}),s=ct(t.date,i),c=rt(t.custom),l=[];return s&&l.push(`date`),Object.keys(c).length>0&&l.push(`custom`),{fieldStates:a,missing:o,dateError:s,customErrors:c,otherProblems:l,canUpload:n&&o.length===0&&l.length===0}}function E(e){return`/details/${encodeURIComponent(e)}`}function ft(e,t){let n=t.find(t=>t.mediatype===e);return n?`${n.mediatype}:${n.collection}`:`texts:${t.find(e=>e.mediatype===`data`)?.collection??`opensource_media`}`}function pt(e){let{mediatype:t,openCollections:n,presetCollection:r,canPickAnyCollection:i}=e,a=ft(t,n),[o]=a.split(`:`),s=r?.includes(`:`)??!1;return i?r?s?r:`${o}:${r}`:a:n.map(e=>`${e.mediatype}:${e.collection}`).find(e=>r?s?e===r:e.endsWith(`:${r}`):e===a)??``}function mt(){return{title:``,identifier:``,description:``,subjects:``,collection:``,creator:``,date:``,language:``,licenseUrl:``,testItem:!1,custom:[]}}function ht(e){let{presets:t,context:n,nextRowId:r}=e,i={...mt(),...t.metadata};return n.languages.some(e=>e.code===i.language)||(i.language=``),n.is_archive_user||(i.testItem=!1),i.custom=t.extraRows.map(e=>({id:r(),key:e.key,value:e.value})),t.presetCollection&&(i.collection=pt({mediatype:void 0,openCollections:n.open_collections,presetCollection:t.presetCollection,canPickAnyCollection:n.can_pick_any_collection})),t.explicitIdentifier&&(i.identifier=t.explicitIdentifier),i}function gt(e){let t=e.replace(/\.[a-zA-Z][a-zA-Z0-9]*$/,``);if(t.split(` `).length===1&&/^[a-z]/.test(t))return t;let n=[];for(let[e]of t.matchAll(/[^ _-]+/g)){let t=0;for(let r of e.matchAll(/[A-Z]+|[0-9][0-9.]*/g))r.index!==0&&(n.push(_t(e.slice(t,r.index))),t=r.index);n.push(_t(e.slice(t)))}return n.join(` `)}function _t(e){return e.charAt(0).toUpperCase()+e.slice(1)}function vt(e){let{metadata:t,largestFile:n,collectionEdited:r,context:i}=e,a={},{titleSuggestion:o}=e;if(!n)return{changes:a,titleSuggestion:o};if((t.title.trim()===``||t.title===o)&&(o=gt(n.file.name),o!==t.title&&(a.title=o)),!r){let r=pt({mediatype:ze(n.file),openCollections:i.open_collections,presetCollection:e.presetCollection,canPickAnyCollection:i.can_pick_any_collection});r!==t.collection&&(a.collection=r)}return{changes:a,titleSuggestion:o}}function yt(e){let{largestFile:t,context:n}=e;return t?pt({mediatype:ze(t.file),openCollections:n.open_collections,canPickAnyCollection:n.can_pick_any_collection}):``}function bt(e){switch(e.stage){case`checkingIdentifier`:case`uploading`:return!0;case`transferFailed`:return!e.nothingUploaded;default:return!1}}var xt=200;function St(e){let{httpStatus:t=0,errorCode:n=``,errorMessage:r=``}=e,i=t>=400&&t<500,a=r.trim();if(!i||a===``)return``;let o=e=>e.replace(/[^a-z]/gi,``).toLowerCase();return n!==``&&o(a)===o(n)?``:a.length>xt?`${a.slice(0,xt-1).trimEnd()}…`:a}function Ct(e,t){if(!e)return a(`The connection to the upload server was lost before it answered.`);let n=String(e),i=t??``,o=i!==``;return a(e===401||e===403?o?r`The upload server refused the file (HTTP status ${n}, ${i}).`:r`The upload server refused the file (HTTP status ${n}).`:e>=500?o?r`The upload server had a problem (HTTP status ${n}, ${i}).`:r`The upload server had a problem (HTTP status ${n}).`:o?r`The upload server answered with HTTP status ${n} (${i}).`:r`The upload server answered with HTTP status ${n}.`)}function wt(e,t){let n=t??``,i=n!==``;if(!e)return i?a(r`The upload server wouldn't accept it: ${n}`):Ct(void 0,void 0);if(e===403)return a(`Your account can't upload with this page right now.`);if(e===401)return a(`You're no longer logged in to archive.org.`);if(!(e>=400&&e<500))return Ct(e,void 0);let o=String(e);return a(i?r`The upload server wouldn't accept it (HTTP status ${o}): ${n}`:r`The upload server wouldn't accept it (HTTP status ${o}).`)}function Tt(e){switch(e.failedDuring){case`identifierCheck`:{let{identifier:t}=e;return a(r`Couldn't confirm that the Page URL ${t} is still available, so nothing was uploaded.`)}case`signing`:return wt(e.httpStatus,e.errorMessage);case`upload`:{let t=Ct(e.httpStatus,e.errorCode),n=St(e);return n===``?t:a(r`${t} It said: ${n}`)}}}function Et(e){let{retryable:t,nothingUploaded:n}=e;return e.failedDuring===`identifierCheck`?a(`Check your connection, then try again.`):a(t?n?`Nothing was uploaded yet. Resuming starts again with this file.`:`Files already sent are kept. Resuming sends this file again from the start, then the rest.`:n?`Nothing was uploaded. Go back to the form to change something; everything you entered is kept.`:`Files already sent are kept in your item, but the rest can't be sent from here.`)}function Dt(e,t){if(t<=0)return 0;let n=Math.min(Math.max(e/t,0),1)*100;return Math.floor(n/10)*10}function Ot(e,t,n={}){switch(e.stage){case`idle`:case`identifierTaken`:return``;case`checkingIdentifier`:return a(`Checking the Page URL before uploading…`);case`uploading`:return a(r`Uploading your files: ${Dt(e.bytesSent,e.bytesTotal)}% sent.`);case`transferFailed`:{let n=Tt(e);if(e.failedDuring===`identifierCheck`)return a(r`${n} Use Try again, or go back to the form.`);let i=t[e.fileIndex]?.path??``;return e.retryable?a(r`Uploading stopped: ${i} couldn't be sent. Use Resume uploading to try again.`):a(r`Uploading stopped: ${i} couldn't be sent. ${n}`)}case`processing`:return e.taskState===`running`?a(`All files sent. Processing…`):a(`All files sent. Waiting to be processed…`);case`taskFailed`:return a(`A processing task for your item failed.`);case`statusUnavailable`:return a(`All files sent. Processing can't be checked from here, but it carries on.`);case`ready`:return n.redirectCancelled?a(`Your item is ready. You'll stay on this page.`):a(r`Your item is ready! Taking you to it in ${10} seconds.`)}}var kt=`/labs/upload`;function At(e){let t=e.replace(/^\?/,``);return t===``?kt:`${kt}?${t}`}function jt(e){let t=[];return e.isLastFile||t.push([`x-archive-queue-derive`,`0`]),t.push([`x-archive-interactive-priority`,`1`]),t.push([`x-archive-size-hint`,String(e.totalBytes)]),t}var Mt=[{status:403,errorCode:`AccessDenied`},{status:400,errorCode:`InvalidBucketName`},{status:409,errorCode:`BucketAlreadyExists`}],Nt=`you lack sufficient privileges to write to this item`;function Pt(e){let t=(e??``).trim().toLowerCase();return t===``||t.startsWith(Nt)}function Ft(e){return Mt.some(t=>t.status===e.status&&t.errorCode===e.errorCode&&(t.errorCode!==`AccessDenied`||Pt(e.errorMessage)))}function It(e){return!e||!(e>=400&&e<500)?!0:e===408||e===429}function Lt(e){return It(e.status)||Ft(e)}function Rt(e){let t=e instanceof v,n=t?e.status:void 0;return{failedDuring:`signing`,retryable:It(n),httpStatus:n,errorMessage:t?e.message:void 0}}function zt(e){return{failedDuring:`upload`,retryable:Lt(e),httpStatus:e.status||void 0,errorCode:e.errorCode,errorMessage:e.errorMessage}}function Bt(e,t){return`/${e}${encodeURIComponent(`/${t}`)}`}var Vt=2e3,Ht=250,Ut=3e4,Wt=5,Gt=3,Kt=class{constructor(e){if(this.options=e,this.currentState={stage:`idle`},this.abortController=new AbortController,this.cancelled=!1,this.identifierConfirmed=!1,this.anyFileSent=!1,this.lastReportAt=-1/0,e.files.length===0)throw Error(`An upload run needs at least one file`);this.bytesTotal=e.files.reduce((e,t)=>e+t.file.size,0),this.pollIntervalMs=e.pollIntervalMs??Vt,this.progressIntervalMs=e.progressIntervalMs??Ht}get state(){return this.currentState}async start(){this.cancelled||this.currentState.stage!==`idle`||await this.runFrom(0)}async resume(){let e=this.currentState,t=e.stage===`transferFailed`&&e.retryable;this.cancelled||!t||await this.runFrom(e.fileIndex)}cancel(){this.cancelled||(this.cancelled=!0,this.abortController.abort(),clearTimeout(this.pollTimer),clearTimeout(this.progressTimer),this.endPollWait?.())}get fileCount(){return this.options.files.length}async runFrom(e){!this.identifierConfirmed&&!await this.confirmIdentifier()||await this.sendFilesFrom(e)&&(this.sendStats({event:`upload`,file_count:this.fileCount,total_bytes:this.bytesTotal}),await this.pollTask())}async confirmIdentifier(){let{identifier:e}=this.options;this.setState({stage:`checkingIdentifier`,identifier:e});let t=await this.checkIdentifier();if(this.cancelled)return!1;let n=t?.check_failed===!0&&t.legal&&!t.available&&t.suggestions.length===0;return!t||n?(this.fail(0,{failedDuring:`identifierCheck`,retryable:!0}),!1):t.available?(this.identifierConfirmed=!0,!0):(this.setState({stage:`identifierTaken`,identifier:e,suggestions:t.suggestions}),!1)}async sendFilesFrom(e){let{files:t}=this.options,n=t.slice(0,e).reduce((e,t)=>e+t.file.size,0);for(let r=e;r<t.length;r++){if(!await this.sendFile(r,n))return!1;n+=t[r].file.size}return!0}async sendFile(e,t){let{identifier:n,files:r,uploadService:i,transferService:a}=this.options,o=r[e],s=o.file.size,c=r=>{if(this.cancelled)return;let i=Math.min(r,s),a=this.currentState;a.stage===`uploading`&&a.fileIndex===e&&a.fileBytesSent===i||this.setState({stage:`uploading`,identifier:n,fileIndex:e,fileCount:this.fileCount,fileBytesSent:i,fileBytesTotal:s,bytesSent:t+i,bytesTotal:this.bytesTotal})};c(0);let l;try{l=await i.signPut({identifier:n,resource:Bt(n,o.path),content_type:o.file.type})}catch(t){return this.cancelled||this.fail(e,Rt(t)),!1}if(this.cancelled)return!1;let u=e===r.length-1,d=[...Object.entries(l.headers),...this.options.metadataHeaders,...jt({isLastFile:u,totalBytes:this.bytesTotal})],f;try{f=await a.put({url:l.url,headers:d,body:o.file,onProgress:e=>c(e),signal:this.abortController.signal})}catch{return this.cancelled||this.fail(e,{failedDuring:`upload`,retryable:!0}),!1}return this.cancelled?!1:f.ok?(this.anyFileSent=!0,c(s),!0):(this.sendStats({event:`error`,http_status:f.status}),e===0&&Ft(f)?await this.handleIdentifierConflict(f):this.fail(e,zt(f)),!1)}async handleIdentifierConflict(e){let t=await this.checkIdentifier();if(this.cancelled)return;let n=t?.available===!0;if(!t||n){this.fail(0,zt(e));return}this.setState({stage:`identifierTaken`,identifier:this.options.identifier,suggestions:t.suggestions})}async checkIdentifier(){let{identifier:e,uploadService:t,maxIdentifierLength:n}=this.options;try{return await t.checkIdentifier(e,{maxLength:n===void 0?void 0:Pe(n),maxSuggestions:Gt})}catch{return}}async pollTask(){let{identifier:e,uploadService:t}=this.options;this.setState({stage:`processing`,identifier:e,taskState:`queued`});let n=0;for(;!this.cancelled;){let r,i=!1;try{r=await t.getStatus(e),n=0}catch(e){n+=1,i=e instanceof v&&e.status!==void 0&&!It(e.status)}if(this.cancelled)return;if(i||n>=Wt){this.setState({stage:`statusUnavailable`,identifier:e});return}let a=r?.state;if(a===`failed`){this.setState({stage:`taskFailed`,identifier:e});return}if(a===`ready`){this.setState({stage:`ready`,identifier:e});return}let o=this.currentState;(a===`queued`||a===`running`)&&!(o.stage===`processing`&&o.taskState===a)&&this.setState({stage:`processing`,identifier:e,taskState:a}),await this.waitBeforePoll(n)}}waitBeforePoll(e){let t=Math.min(this.pollIntervalMs*2**e,Math.max(Ut,this.pollIntervalMs));return new Promise(e=>{this.endPollWait=e,this.pollTimer=setTimeout(e,t)})}fail(e,t){let{httpStatus:n,errorCode:r,errorMessage:i}=t;this.setState({stage:`transferFailed`,identifier:this.options.identifier,fileIndex:e,fileCount:this.fileCount,failedDuring:t.failedDuring,retryable:t.retryable,nothingUploaded:!this.anyFileSent,...n?{httpStatus:n}:{},...r?{errorCode:r}:{},...i?{errorMessage:i}:{}})}sendStats(e){this.options.uploadService.sendStats(e).catch(()=>{})}setState(e){let t=this.currentState.stage===`uploading`&&e.stage===`uploading`;this.currentState=e,clearTimeout(this.progressTimer),this.progressTimer=void 0;let n=this.lastReportAt+this.progressIntervalMs-performance.now();if(!t||n<=0){this.report();return}this.progressTimer=setTimeout(()=>{this.progressTimer=void 0,this.report()},n)}report(){this.lastReportAt=performance.now(),this.options.onStateChange(this.currentState)}};function D(e,t){return new CustomEvent(e,{detail:t})}function qt(){let e=typeof HTMLInputElement<`u`&&`webkitdirectory`in HTMLInputElement.prototype,t=typeof navigator>`u`?void 0:navigator;return{inputSupportsDirectories:e,userAgent:t?.userAgent??``,maxTouchPoints:t?.maxTouchPoints??0}}function Jt(e=qt()){return e.inputSupportsDirectories?!Yt(e):!1}function Yt(e){let{userAgent:t,maxTouchPoints:n}=e,r=/\b(iPhone|iPad|iPod)\b/.test(t),i=/\bMacintosh\b/.test(t)&&n>1;return r||i}async function Xt(e){let t=e.createReader(),n=[],r;do r=await Zt(t),n.push(...r);while(r.length>0);return n}function Zt(e){return new Promise((t,n)=>{e.readEntries(t,n)})}async function Qt(e){return(await Promise.all(e.map(e=>$t(e,``)))).flat()}async function $t(e,t){let n=en(e,t);if(tn(e))return[{file:await rn(e),path:n}];if(nn(e)){let t=await Xt(e);return(await Promise.all(t.map(e=>$t(e,n)))).flat()}return[]}function en(e,t){return e.fullPath.replace(/^\/+/,``)||(t?`${t}/${e.name}`:e.name)}function tn(e){return e.isFile}function nn(e){return e.isDirectory}function rn(e){return new Promise((t,n)=>{e.file(t,n)})}function an(e){return sn(on(e))}function on(e){let t=[];for(let n of Array.from(e.items??[])){if(n.kind!==`file`)continue;let e=(typeof n.webkitGetAsEntry==`function`?n.webkitGetAsEntry():null)??n.getAsFile();e&&t.push(e)}return t.length===0&&t.push(...Array.from(e.files??[])),t}async function sn(e){let t=await Promise.allSettled(e.map(async e=>e instanceof File?[{file:e,path:e.name}]:Qt([e]))),n=[],r=[];for(let e of t)e.status===`fulfilled`?n.push(...e.value):r.push(e.reason);return{candidates:n,errors:r}}function cn(e){let t=x(e.fileCount);if(e.fileCount===0)return t;let n=C(e.totalBytes);return e.folderCount===0?a(r`${t} · ${n}`):a(r`${t} in ${ne(e.folderCount)} · ${n}`)}function ln(e){return e===0?``:a(e===1?`1 system file skipped`:r`${e.toLocaleString()} system files skipped`)}function un(e,t=200){let n=[],r=[];if(dn(e,n,r),n.reduce((e,t)=>e+t,0)<=t)return[];let i=0,a=0;for(let[e,r]of n.entries()){if(a+=r,a>t)break;i=e}return r.filter(e=>e.depth>=i).map(e=>e.id)}function dn(e,t,n){for(let r of e)t[r.depth]=(t[r.depth]??0)+1,r.kind===`folder`&&(n.push(r),dn(r.children,t,n))}var fn={collapsed:new Set,seen:new Set};function O(e){return e.flatMap(e=>e.kind===`folder`?[e.id,...O(e.children)]:[])}function pn(e,t){let n=new Set(O(e)),r=new Set([...t.collapsed].filter(e=>n.has(e)));for(let n of un(e))t.seen.has(n)||r.add(n);return{collapsed:r,seen:n}}var k=e`
  :host {
    --upload-drop-zone-fill: color-mix(
      in srgb,
      var(--ia-theme-link-color, #4b64ff) 10%,
      var(--ia-theme-secondary-background-color, #fff)
    );
    --upload-drop-zone-dash: color-mix(
      in srgb,
      var(--ia-theme-link-color, #4b64ff) 70%,
      var(--ia-theme-secondary-background-color, #fff)
    );
  }

  *,
  *::before,
  *::after {
    box-sizing: border-box;
  }

  :focus-visible {
    outline: 2px solid var(--ia-theme-link-color, #4b64ff);
    outline-offset: 2px;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
`;function A(e){return e?Array.from(e.types??[]).includes(`Files`):!1}var j=1;function mn(e){let{scrollTop:t,scrollHeight:n,clientHeight:r}=e,i=n-r>j;return i?{overflows:i,moreAbove:t>j,moreBelow:t+r<n-j}:{overflows:i,moreAbove:!1,moreBelow:!1}}function hn(e,t){if(!(t===``||!e.startsWith(t)))return[t,e.slice(t.length)]}var gn=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M8 2.5v11M2.5 8h11"
    fill="none"
    stroke="currentColor"
    stroke-width="2.2"
    stroke-linecap="round"
  />
</svg>`,_n=i`<svg
  class="chevron"
  viewBox="0 0 12 8"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M1 1.5l5 5 5-5"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>`,vn=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M3.5 1.5h6l3 3v10h-9Z"
    fill="none"
    stroke="currentColor"
    stroke-width="1.5"
    stroke-linejoin="round"
  />
  <path
    d="M9.5 1.5v3h3"
    fill="none"
    stroke="currentColor"
    stroke-width="1.5"
    stroke-linejoin="round"
  />
</svg>`,yn=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M1.5 4.2c0-.7.5-1.2 1.2-1.2h3.2l1.6 1.7h5.8c.7 0 1.2.5 1.2 1.2v6.4c0 .7-.5 1.2-1.2 1.2H2.7c-.7 0-1.2-.5-1.2-1.2Z"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linejoin="round"
  />
</svg>`,bn=i`<svg
  class="disclosure"
  viewBox="0 0 10 10"
  aria-hidden="true"
  focusable="false"
>
  <path d="M2 .6 9.2 5 2 9.4Z" fill="currentColor" />
</svg>`,xn=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M3.5 3.5l9 9M12.5 3.5l-9 9"
    fill="none"
    stroke="currentColor"
    stroke-width="2.2"
    stroke-linecap="round"
  />
</svg>`,M=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M8 1.3 15.3 14.4H.7Z"
    fill="currentColor"
    stroke="currentColor"
    stroke-width=".8"
    stroke-linejoin="round"
  />
  <path
    class="cutout"
    d="M8 6.1v3.7"
    fill="none"
    stroke-width="1.8"
    stroke-linecap="round"
  />
  <circle class="cutout-fill" cx="8" cy="12" r="1" />
</svg>`,Sn=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <circle cx="8" cy="8" r="7.4" fill="currentColor" />
  <path
    class="cutout"
    d="M4.5 8.3l2.3 2.3 4.7-4.9"
    fill="none"
    stroke-width="1.9"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>`,Cn=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <circle
    cx="8"
    cy="8"
    r="6.6"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
  />
  <path
    d="M8 4.6V8l2.4 1.6"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>`,N=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M8 13.5V3M3.5 7.2 8 2.7l4.5 4.5"
    fill="none"
    stroke="currentColor"
    stroke-width="1.9"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>`,wn=i`<svg
  class="tree-icon folder-icon"
  viewBox="0 0 28 28"
  aria-hidden="true"
  focusable="false"
>
  <path
    class="folder-front"
    d="M3 8c0-1 .8-1.8 1.8-1.8h6l2.4 2.6h10c1 0 1.8.8 1.8 1.8V22c0 1-.8 1.8-1.8 1.8H4.8C3.8 23.8 3 23 3 22Z"
    stroke-width="1.4"
    stroke-linejoin="round"
  />
</svg>`,Tn=i`<svg
  class="tree-icon folder-icon"
  viewBox="0 0 28 28"
  aria-hidden="true"
  focusable="false"
>
  <path
    class="folder-back"
    d="M6 5.5c0-.8.6-1.4 1.4-1.4h4.8l1.9 2h8.5c.8 0 1.4.6 1.4 1.4"
    fill="none"
    stroke-width="1.2"
    stroke-linejoin="round"
  />
  <path
    class="folder-front"
    d="M3 10c0-1 .8-1.8 1.8-1.8h6l2.4 2.6h10c1 0 1.8.8 1.8 1.8V22c0 1-.8 1.8-1.8 1.8H4.8C3.8 23.8 3 23 3 22Z"
    stroke-width="1.4"
    stroke-linejoin="round"
  />
</svg>`;function En(e){return i`<svg
    class="tree-icon file-icon"
    viewBox="0 0 28 28"
    aria-hidden="true"
    focusable="false"
  >
    <path
      class="page"
      d="M6.5 2.5h10.5l5 5v18h-15.5Z"
      stroke-width="1.2"
      stroke-linejoin="round"
    />
    <path
      class="page-fold"
      d="M17 2.5v5h5"
      fill="none"
      stroke-width="1.2"
      stroke-linejoin="round"
    />
    ${e===``?n:c`<rect class="band" x="2.5" y="14" width="17.5" height="8" rx="1" />
          <text
            class="band-text"
            x="11.25"
            y=${e.length>3?19.8:20.1}
            text-anchor="middle"
            font-size=${e.length>3?5:6}
            font-weight="700"
          >${e}</text>`}
  </svg>`}var Dn=e`
  svg .cutout {
    stroke: var(--ia-theme-secondary-background-color, #fff);
  }

  svg .cutout-fill {
    fill: var(--ia-theme-secondary-background-color, #fff);
  }

  .tree-icon {
    display: block;
    width: 28px;
    height: 28px;
  }

  .file-icon .page {
    fill: var(--ia-theme-secondary-background-color, #fff);
    stroke: var(--ia-theme-field-border-color, #767676);
  }

  .file-icon .page-fold {
    stroke: var(--ia-theme-field-border-color, #767676);
  }

  /* The band stays dark grey with white text in both themes (7:1). */
  .file-icon .band {
    fill: #595959;
  }

  .file-icon .band-text {
    fill: #fff;
    font-family: Helvetica, Arial, sans-serif;
  }

  .folder-icon .folder-front {
    fill: var(--ia-theme-tertiary-background-color, #efefef);
    stroke: var(--ia-theme-secondary-text-color, #666);
  }

  .folder-icon .folder-back {
    stroke: var(--ia-theme-field-border-color, #767676);
  }
`;function On(e){return Array.from(e,e=>({file:e,path:kn(e)}))}function kn(e){return(e.webkitRelativePath??``).replace(/^\/+/,``)||e.name}var P=class extends s{constructor(...e){super(...e),this.hasFiles=!1,this.filesOnly=!1,this.open=!1,this.onDocumentClick=e=>{e.composedPath().includes(this)||this.closeMenu(!1)}}focusButton(){this.addButton?.focus()}disconnectedCallback(){super.disconnectedCallback(),this.open=!1,document.removeEventListener(`click`,this.onDocumentClick)}updated(e){e.has(`open`)&&(this.open?document.addEventListener(`click`,this.onDocumentClick):document.removeEventListener(`click`,this.onDocumentClick))}render(){let e={"add-button":!0,primary:!this.hasFiles,secondary:this.hasFiles};return i`
      <div
        class="add-wrap"
        @keydown=${this.onKeydown}
        @focusout=${this.onFocusOut}
      >
        <button
          type="button"
          id="add-button"
          class=${l(e)}
          aria-expanded=${this.open?`true`:`false`}
          aria-controls="add-menu"
          @click=${this.toggleMenu}
        >
          ${gn}${a(`Add files or folders`)}${_n}
        </button>
        <div id="add-menu" class="menu" ?hidden=${!this.open}>
          <button
            type="button"
            id="choose-files"
            class="menu-item"
            aria-labelledby="choose-files-label"
            aria-describedby="choose-files-hint"
            @click=${this.chooseFiles}
          >
            ${vn}
            <span class="item-text">
              <span class="item-label" id="choose-files-label"
                >${a(`Choose files…`)}</span
              >
              <span class="hint" id="choose-files-hint"
                >${a(`One or more files, from any folder`)}</span
              >
            </span>
          </button>
          ${this.filesOnly?n:this.folderItemTemplate}
        </div>
        <input
          id="files-input"
          type="file"
          multiple
          hidden
          @change=${this.onInputChange}
        />
        ${this.filesOnly?n:i`<input
              id="folder-input"
              type="file"
              webkitdirectory
              hidden
              @change=${this.onInputChange}
            />`}
      </div>
    `}get folderItemTemplate(){return i`<button
      type="button"
      id="choose-folder"
      class="menu-item"
      aria-labelledby="choose-folder-label"
      aria-describedby="choose-folder-hint"
      @click=${this.chooseFolder}
    >
      ${yn}
      <span class="item-text">
        <span class="item-label" id="choose-folder-label"
          >${a(`Choose a folder…`)}</span
        >
        <span class="hint" id="choose-folder-hint"
          >${a(`Everything inside, subfolders included`)}</span
        >
      </span>
    </button>`}async toggleMenu(){this.open=!this.open,this.open&&(await this.updateComplete,this.firstItem?.focus())}closeMenu(e){this.open=!1,e&&this.focusButton()}chooseFiles(){this.closeMenu(!0),this.filesInput?.click()}chooseFolder(){this.closeMenu(!0),this.folderInput?.click()}onKeydown(e){e.key!==`Escape`||!this.open||(e.preventDefault(),this.closeMenu(!0))}onFocusOut(e){let t=e.relatedTarget;!this.open||!(t instanceof Node)||this.renderRoot.contains(t)||this.closeMenu(!1)}onInputChange(e){let t=e.target;if(!(t instanceof HTMLInputElement))return;let n=On(t.files??[]);t.value=``,this.dispatchEvent(D(`filesPicked`,{candidates:n}))}static get styles(){return[k,e`
        :host {
          display: inline-block;
        }

        .add-wrap {
          position: relative;
        }

        .add-button {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          gap: 8px;
          height: 40px;
          padding: 0 18px;
          border: 1px solid transparent;
          border-radius: 4px;
          font: inherit;
          font-size: 15px;
          font-weight: 700;
          line-height: 1;
          white-space: nowrap;
          cursor: pointer;
        }

        .add-button .glyph {
          width: 14px;
          height: 14px;
          flex: none;
        }

        .add-button .chevron {
          width: 11px;
          height: 8px;
          margin-left: 2px;
          flex: none;
        }

        /* #4b64ff with white text in both themes: the dark theme's link
           colour is too light to sit behind white text. */
        .primary {
          background: var(--upload-primary-button-fill, #4b64ff);
          border-color: var(--upload-primary-button-border, #3b52e6);
          color: #fff;
        }

        .primary:hover {
          background: var(--upload-primary-button-hover-fill, #3b52e6);
        }

        /* A white outline button in both themes: #194880 text needs a light
           surface to meet contrast. */
        .secondary {
          background: #fff;
          border-color: var(--ia-theme-primary-cta-fill, #194880);
          color: var(--ia-theme-primary-cta-fill, #194880);
        }

        .secondary:hover {
          background: #edf1ff;
        }

        .menu {
          position: absolute;
          z-index: 5;
          top: calc(100% + 6px);
          left: 50%;
          transform: translateX(-50%);
          display: flex;
          flex-direction: column;
          gap: 2px;
          min-width: 250px;
          padding: 6px;
          background: var(--ia-theme-secondary-background-color, #fff);
          border: 1px solid var(--ia-theme-border-color, #ccc);
          border-radius: 4px;
          box-shadow: 0 8px 22px rgba(0, 0, 0, 0.16);
          text-align: left;
        }

        .menu[hidden] {
          display: none;
        }

        .menu-item {
          display: flex;
          align-items: flex-start;
          gap: 10px;
          width: 100%;
          padding: 8px 10px;
          border: 0;
          border-radius: 3px;
          background: none;
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          font: inherit;
          text-align: left;
          cursor: pointer;
        }

        .menu-item:hover {
          background: var(--upload-drop-zone-fill);
        }

        .menu-item .glyph {
          width: 16px;
          height: 16px;
          flex: none;
          margin-top: 2px;
          color: var(--ia-theme-link-color, #4b64ff);
        }

        .item-label {
          display: block;
          font-size: 14px;
          font-weight: 700;
        }

        .hint {
          display: block;
          font-size: 13px;
          color: var(--ia-theme-secondary-text-color, #666);
        }
      `]}};_([t({type:Boolean,reflect:!0})],P.prototype,`hasFiles`,void 0),_([t({type:Boolean})],P.prototype,`filesOnly`,void 0),_([o()],P.prototype,`open`,void 0),_([m(`#add-button`)],P.prototype,`addButton`,void 0),_([m(`#choose-files`)],P.prototype,`firstItem`,void 0),_([m(`#files-input`)],P.prototype,`filesInput`,void 0),_([m(`#folder-input`)],P.prototype,`folderInput`,void 0),P=_([p(`upload-add-menu`)],P);var An=20*1024*1024;async function jn(e,t=An){if(!e.type.toLowerCase().startsWith(`image/`)||e.size===0||e.size>t||typeof createImageBitmap!=`function`)return!1;try{return(await createImageBitmap(e)).close(),!0}catch{return!1}}function Mn(e){let t=e.slice(e.lastIndexOf(`/`)+1),n=t.lastIndexOf(`.`);if(n<=0)return``;let r=t.slice(n+1);return/^[a-z0-9]{1,10}$/i.test(r)?r.toLowerCase():``}function Nn(e){return Mn(e).toUpperCase().slice(0,4)}var Pn=class{constructor(){this.urls=new Map}urlFor(e){let t=this.urls.get(e.id);if(t)return t;let n=URL.createObjectURL(e.file);return this.urls.set(e.id,n),n}revoke(e){for(let t of e){let e=this.urls.get(t.id);e!==void 0&&(URL.revokeObjectURL(e),this.urls.delete(t.id))}}revokeAll(){for(let e of this.urls.values())URL.revokeObjectURL(e);this.urls.clear()}},Fn=[{mimeTypes:[`image/jpeg`,`image/pjpeg`],extensions:[`jpg`,`jpeg`,`jpe`,`jfif`],label:()=>a(`JPEG image`)},{mimeTypes:[`image/png`],extensions:[`png`],label:()=>a(`PNG image`)},{mimeTypes:[`image/gif`],extensions:[`gif`],label:()=>a(`GIF image`)},{mimeTypes:[`image/webp`],extensions:[`webp`],label:()=>a(`WebP image`)},{mimeTypes:[`image/tiff`],extensions:[`tif`,`tiff`],label:()=>a(`TIFF image`)},{mimeTypes:[`image/jp2`,`image/jpx`],extensions:[`jp2`,`jpx`,`j2k`],label:()=>a(`JPEG 2000 image`)},{mimeTypes:[`image/svg+xml`],extensions:[`svg`],label:()=>a(`SVG image`)},{mimeTypes:[`image/heic`,`image/heif`],extensions:[`heic`,`heif`],label:()=>a(`HEIC image`)},{mimeTypes:[`application/pdf`],extensions:[`pdf`],label:()=>a(`PDF document`)},{mimeTypes:[`application/epub+zip`],extensions:[`epub`],label:()=>a(`EPUB book`)},{mimeTypes:[`image/vnd.djvu`],extensions:[`djvu`,`djv`],label:()=>a(`DjVu document`)},{mimeTypes:[`application/msword`,`application/vnd.openxmlformats-officedocument.wordprocessingml.document`],extensions:[`doc`,`docx`],label:()=>a(`Word document`)},{mimeTypes:[`text/plain`],extensions:[`txt`],label:()=>a(`Plain text`)},{mimeTypes:[`text/html`],extensions:[`html`,`htm`],label:()=>a(`HTML document`)},{mimeTypes:[`text/csv`],extensions:[`csv`],label:()=>a(`CSV data`)},{mimeTypes:[`application/json`],extensions:[`json`],label:()=>a(`JSON data`)},{mimeTypes:[`application/xml`,`text/xml`],extensions:[`xml`],label:()=>a(`XML document`)},{mimeTypes:[`audio/mpeg`],extensions:[`mp3`],label:()=>a(`MP3 audio`)},{mimeTypes:[`audio/flac`,`audio/x-flac`],extensions:[`flac`],label:()=>a(`FLAC audio`)},{mimeTypes:[`audio/wav`,`audio/x-wav`,`audio/wave`,`audio/vnd.wave`],extensions:[`wav`],label:()=>a(`WAV audio`)},{mimeTypes:[`audio/ogg`,`audio/vorbis`],extensions:[`ogg`,`oga`],label:()=>a(`Ogg audio`)},{mimeTypes:[`audio/aiff`,`audio/x-aiff`],extensions:[`aif`,`aiff`],label:()=>a(`AIFF audio`)},{mimeTypes:[`audio/mp4`,`audio/x-m4a`,`audio/aac`],extensions:[`m4a`,`aac`],label:()=>a(`AAC audio`)},{mimeTypes:[`video/mp4`],extensions:[`mp4`,`m4v`],label:()=>a(`MP4 video`)},{mimeTypes:[`video/quicktime`],extensions:[`mov`],label:()=>a(`QuickTime video`)},{mimeTypes:[`video/webm`],extensions:[`webm`],label:()=>a(`WebM video`)},{mimeTypes:[`video/x-matroska`],extensions:[`mkv`],label:()=>a(`Matroska video`)},{mimeTypes:[`video/x-msvideo`,`video/avi`],extensions:[`avi`],label:()=>a(`AVI video`)},{mimeTypes:[`video/mpeg`],extensions:[`mpg`,`mpeg`],label:()=>a(`MPEG video`)},{mimeTypes:[`application/zip`,`application/x-zip-compressed`],extensions:[`zip`],label:()=>a(`ZIP archive`)},{mimeTypes:[`application/x-tar`],extensions:[`tar`],label:()=>a(`TAR archive`)},{mimeTypes:[`application/gzip`,`application/x-gzip`],extensions:[`gz`,`tgz`],label:()=>a(`Gzip archive`)},{mimeTypes:[`application/x-iso9660-image`],extensions:[`iso`],label:()=>a(`Disc image`)}];function In(e){let t=e.type.split(`;`)[0].trim().toLowerCase(),n=Mn(e.name),i=Fn.find(e=>e.mimeTypes.includes(t))??Fn.find(e=>e.extensions.includes(n));if(i)return i.label();let o=n.toUpperCase();switch(t.includes(`/`)?t.split(`/`)[0]:``){case`image`:return a(o?r`${o} image`:`Image`);case`audio`:return a(o?r`${o} audio`:`Audio`);case`video`:return a(o?r`${o} video`:`Video`);default:return a(o?r`${o} file`:`File`)}}function Ln(e){return e.kind===`file`?e.name:e.names.join(` / `)}function Rn(e){return Un([e.nestedFolderCount>1?a(r`${e.nestedFolderCount} nested folders`):void 0,x(e.fileCount),C(e.totalBytes),Hn(e)])}function zn(e,t){return Un([C(e.size),In(e.uploadFile.file),t?void 0:a(`no preview`),Hn(e)])}function Bn(e){let t=Ln(e);return e.kind===`file`?a(r`Remove ${t}`):e.fileCount===1?a(r`Remove folder ${t} and its 1 file`):a(r`Remove folder ${t} and its ${e.fileCount.toLocaleString()} files`)}function Vn(e){return a(r`${Ln(e)} folder`)}function Hn(e){return e.indentCapped?a(r`level ${e.level}`):void 0}function Un(e){let t=a(` · `,{desc:`Separates the parts of a file-list row, e.g. "490 KB · JPEG image".`});return e.filter(e=>e!==void 0).join(t)}function Wn(e){let t=new Map,n=e=>{for(let r of e)r.kind===`folder`?n(r.children):t.set(r.id,r.uploadFile)};return n(e),t}function Gn(e,t){let n=new Map,r=e=>{let i={total:0,done:0,failed:0},a=!1;for(let o of e){if(o.kind===`folder`){let e=r(o.children);i.total+=e.summary.total,i.done+=e.summary.done,i.failed+=e.summary.failed,e.inRun&&(a=!0,n.set(o.id,e.summary));continue}i.total+=1;let e=t[o.id];e&&(a=!0,e.state===`done`&&(i.done+=1),e.state===`failed`&&(i.failed+=1))}return{summary:i,inRun:a}};return r(e),n}function Kn(e,t=An){return e.type.toLowerCase().startsWith(`image/`)&&e.size>0&&e.size<=t}function qn(e){return Number.isFinite(e)?Math.min(100,Math.max(0,Math.floor(e*100))):0}function Jn(e){switch(e.state){case`waiting`:return a(`Waiting`);case`uploading`:return a(r`Uploading ${qn(e.fraction)}%`);case`done`:return a(`Uploaded`);case`failed`:return a(`Failed`)}}function Yn(e){let t=a(r`${e.done.toLocaleString()} of ${e.total.toLocaleString()} uploaded`);if(e.failed===0)return t;let n=a(r`${e.failed.toLocaleString()} failed`);return`${t}${a(` · `,{desc:`Separates the parts of a file-list row, e.g. "490 KB · JPEG image".`})}${n}`}var F=class extends s{constructor(...e){super(...e),this.rows=[],this.collapsedIds=new Set,this.transferStatus={},this.disabled=!1,this.previewResults=new Map,this.folderSummaries=new Map,this.previewUrls=new Pn,this.knownFiles=new Map,this.checking=new Set,this.observedRows=new Set,this.onIntersection=e=>{for(let t of e){if(!t.isIntersecting)continue;this.observer?.unobserve(t.target),this.observedRows.delete(t.target);let e=t.target instanceof HTMLElement?t.target.dataset.fileId:void 0;e!==void 0&&this.checkPreview(e)}}}disconnectedCallback(){super.disconnectedCallback(),this.observer?.disconnect(),this.observer=void 0,this.observedRows.clear(),this.previewUrls.revokeAll(),this.checking.clear(),this.previewResults=new Map}willUpdate(e){e.has(`rows`)&&this.syncFiles(),(e.has(`rows`)||e.has(`transferStatus`))&&(this.folderSummaries=Gn(this.rows,this.transferStatus))}updated(e){e.size===1&&e.has(`transferStatus`)||this.watchRowsToCheck(),e.has(`rows`)&&this.restoreFocusAfterRemove()}render(){return i`<ul class="tree" @click=${this.onClick}>
      ${this.rowsTemplate(this.rows)}
    </ul>`}rowsTemplate(e){return d(e,e=>e.id,e=>e.kind===`folder`?this.folderRowTemplate(e):this.fileRowTemplate(e))}folderRowTemplate(e){let t=!this.collapsedIds.has(e.id),r=`children-${e.id}`,a=e.nestedFolderCount>1,o=this.folderSummaries.get(e.id);return i`<li>
      <div class="node is-folder depth-${e.displayDepth}">
        <button
          type="button"
          class="toggle"
          data-action="toggle"
          data-node-id=${e.id}
          aria-expanded=${t?`true`:`false`}
          aria-controls=${f(t?r:void 0)}
          aria-label=${Vn(e)}
        >
          ${bn}
        </button>
        <span class="icon">${a?Tn:wn}</span>
        <span class="meta">
          <span class="name">${this.folderNameTemplate(e)}</span>
          <span class="details">
            <span>${Rn(e)}</span>
            ${o?this.folderStatusTemplate(o):n}
          </span>
        </span>
        ${this.removeButtonTemplate(e)}
      </div>
      ${t?i`<ul id=${r}>
            ${this.rowsTemplate(e.children)}
          </ul>`:n}
    </li>`}folderNameTemplate(e){return e.names.map((e,t)=>i`${t>0?i`<span class="separator"> / </span>`:n}${e}`)}fileRowTemplate(e){let t=this.previewState(e),r=this.transferStatus[e.id],a=t===`pending`;return i`<li>
      <div
        class="node is-file depth-${e.displayDepth}"
        data-file-id=${e.id}
        ?data-preview-check=${a}
      >
        <span class="lead"></span>
        <span class="icon">${this.fileIconTemplate(e,t)}</span>
        <span class="meta">
          <span class="name">${e.name}</span>
          <span class="details">
            <span>${zn(e,t!==`icon`)}</span>
            ${r?this.fileStatusTemplate(e,r):n}
          </span>
        </span>
        ${this.removeButtonTemplate(e)}
      </div>
    </li>`}fileIconTemplate(e,t){return t===`thumbnail`?i`<span class="thumbnail">
      <img
        src=${this.previewUrls.urlFor(e.uploadFile)}
        alt=""
        width="28"
        height="28"
        decoding="async"
        data-file-id=${e.id}
        @error=${this.onThumbnailError}
      />
    </span>`:En(Nn(e.name))}removeButtonTemplate(e){return this.disabled?n:i`<button
      type="button"
      class="remove"
      data-action="remove"
      data-node-id=${e.id}
      aria-label=${Bn(e)}
    >
      ${xn}
    </button>`}fileStatusTemplate(e,t){let n=Jn(t);switch(t.state){case`waiting`:return i`<span class="status waiting">${Cn}${n}</span>`;case`uploading`:{let o=Ln(e);return i`<span class="status uploading">
          ${N}${n}
          <progress
            max="100"
            value=${qn(t.fraction)}
            aria-label=${a(r`${o} upload progress`)}
          ></progress>
        </span>`}case`done`:return i`<span class="status done">${Sn}${n}</span>`;case`failed`:return i`<span class="status failed">${M}${n}</span>`}}folderStatusTemplate(e){let t=Yn(e);return e.failed>0?i`<span class="status failed">${M}${t}</span>`:e.done===e.total?i`<span class="status done">${Sn}${t}</span>`:i`<span class="status in-progress"
      >${N}${t}</span
    >`}previewState(e){let t=this.previewResults.get(e.id);return t===!0?`thumbnail`:t===!1||!Kn(e.uploadFile.file)?`icon`:`pending`}onClick(e){let t=e.target;if(!(t instanceof Element))return;let n=t.closest(`button[data-action]`);if(!(n instanceof HTMLButtonElement))return;let r=n.dataset.nodeId;if(r===void 0)return;if(n.dataset.action===`toggle`){let e={folderId:r};this.dispatchEvent(new CustomEvent(`folderToggled`,{detail:e}));return}let i=Array.from(this.renderRoot.querySelectorAll(`.remove`));this.focusIndexAfterRemove=i.indexOf(n),this.dispatchEvent(D(`fileNodeRemoved`,{nodeId:r}))}restoreFocusAfterRemove(){let e=this.focusIndexAfterRemove;if(e===void 0)return;this.focusIndexAfterRemove=void 0;let t=this.renderRoot.querySelectorAll(`.remove`);t.length!==0&&t[Math.min(e,t.length-1)]?.focus()}syncFiles(){let e=this.knownFiles,t=Wn(this.rows);this.knownFiles=t;let n=[...e.values()].filter(e=>!t.has(e.id));if(n.length!==0){this.previewUrls.revoke(n);for(let e of n)this.checking.delete(e.id);this.previewResults=new Map([...this.previewResults].filter(([e])=>t.has(e)))}}watchRowsToCheck(){if(typeof IntersectionObserver>`u`)return;let e=this.observer??(this.observer=new IntersectionObserver(this.onIntersection,{rootMargin:`100px 0px`}));for(let t of this.observedRows)t.isConnected&&t.hasAttribute(`data-preview-check`)||(e.unobserve(t),this.observedRows.delete(t));for(let t of this.renderRoot.querySelectorAll(`[data-preview-check]`))this.observedRows.has(t)||(e.observe(t),this.observedRows.add(t))}async checkPreview(e){let t=this.knownFiles.get(e);if(!t||this.checking.has(e)||this.previewResults.has(e))return;this.checking.add(e);let n=await jn(t.file);this.checking.delete(e)&&(this.previewResults=new Map(this.previewResults).set(e,n))}onThumbnailError(e){let t=e.target;if(!(t instanceof HTMLImageElement))return;let n=t.dataset.fileId,r=n===void 0?void 0:this.knownFiles.get(n);!n||!r||(this.previewUrls.revoke([r]),this.previewResults=new Map(this.previewResults).set(n,!1))}static get styles(){return[k,Dn,e`
        :host {
          display: block;
        }

        ul {
          list-style: none;
          margin: 0;
          padding: 0;
        }

        .node {
          --step: 18px;
          --indent: calc(var(--depth, 0) * var(--step));
          --guide: var(--ia-theme-border-color, #ccc);
          display: grid;
          grid-template-columns: 24px 28px minmax(0, 1fr) 28px;
          gap: 8px;
          align-items: center;
          padding: 7px 10px 7px calc(10px + var(--indent));
          border-top: 1px solid var(--ia-theme-border-color, #ccc);
          /* One 1px guide line under each ancestor's disclosure: the first
             sits at 10px padding + half the 24px disclosure. */
          background: repeating-linear-gradient(
              90deg,
              var(--guide) 0 1px,
              transparent 1px var(--step)
            )
            22px 0 / var(--indent) 100% no-repeat;
        }

        :host([disabled]) .node {
          grid-template-columns: 24px 28px minmax(0, 1fr);
        }

        .tree > li:first-child > .node {
          border-top: 0;
        }

        .depth-1 {
          --depth: 1;
        }

        .depth-2 {
          --depth: 2;
        }

        .depth-3 {
          --depth: 3;
        }

        .depth-4 {
          --depth: 4;
        }

        .depth-5 {
          --depth: 5;
        }

        @media (max-width: 767px) {
          .node {
            --step: 14px;
          }
        }

        .toggle {
          display: grid;
          place-items: center;
          width: 24px;
          height: 24px;
          padding: 0;
          border: 0;
          border-radius: 3px;
          background: none;
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          cursor: pointer;
        }

        .toggle:hover {
          background: var(--ia-theme-tertiary-background-color, #efefef);
        }

        .toggle .disclosure {
          width: 10px;
          height: 10px;
          transition: transform 0.15s ease;
        }

        .toggle[aria-expanded='true'] .disclosure {
          transform: rotate(90deg);
        }

        @media (prefers-reduced-motion: reduce) {
          .toggle .disclosure {
            transition: none;
          }
        }

        .icon {
          display: block;
          width: 28px;
          height: 28px;
        }

        .thumbnail {
          position: relative;
          display: block;
          width: 28px;
          height: 28px;
        }

        .thumbnail img {
          display: block;
          width: 28px;
          height: 28px;
          border-radius: 2px;
          object-fit: cover;
        }

        .thumbnail::after {
          content: '';
          position: absolute;
          inset: 0;
          border-radius: 2px;
          box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.14);
        }

        .meta {
          display: flex;
          flex-direction: column;
          min-width: 0;
          line-height: 1.3;
        }

        .name {
          font-size: 14px;
          overflow-wrap: anywhere;
        }

        .is-folder .name {
          font-weight: 700;
        }

        .separator {
          padding: 0 1px;
          color: var(--ia-theme-secondary-text-color, #666);
          font-weight: 400;
        }

        .details {
          display: flex;
          flex-wrap: wrap;
          align-items: center;
          column-gap: 8px;
          font-size: 12px;
          color: var(--ia-theme-secondary-text-color, #666);
          font-variant-numeric: tabular-nums;
        }

        .status {
          display: inline-flex;
          align-items: center;
          gap: 4px;
          font-weight: 700;
        }

        .status .glyph {
          width: 13px;
          height: 13px;
          flex: none;
        }

        .status.waiting {
          color: var(--ia-theme-secondary-text-color, #666);
          font-weight: 400;
        }

        .status.uploading,
        .status.in-progress {
          color: var(--ia-theme-link-color, #4b64ff);
        }

        .status.done {
          color: var(--ia-theme-color-success-text, #267326);
        }

        .status.failed {
          color: var(--ia-theme-color-danger-text, #c62828);
        }

        /* The track is the row's own background inside an input-border
           outline, so both the bar's full extent (4.5:1 against the row)
           and the filled part (4.6:1 against the track) meet 3:1. A grey
           track left the fill at 2.9:1. */
        progress {
          width: 48px;
          height: 8px;
          margin-left: 2px;
          border: 1px solid var(--ia-theme-field-border-color, #767676);
          border-radius: 4px;
          background: var(--ia-theme-secondary-background-color, #fff);
          color: var(--ia-theme-link-color, #4b64ff);
          appearance: none;
        }

        progress::-webkit-progress-bar {
          border-radius: 3px;
          background: var(--ia-theme-secondary-background-color, #fff);
        }

        progress::-webkit-progress-value {
          border-radius: 3px;
          background: var(--ia-theme-link-color, #4b64ff);
        }

        progress::-moz-progress-bar {
          border-radius: 3px;
          background: var(--ia-theme-link-color, #4b64ff);
        }

        .remove {
          display: grid;
          place-items: center;
          width: 28px;
          height: 28px;
          padding: 0;
          /* The input border token, for 3:1 against the row in both
             themes (the design's #999 was 2.85:1 on white). */
          border: 1px solid var(--ia-theme-field-border-color, #767676);
          border-radius: 3px;
          background: var(--ia-theme-secondary-background-color, #fff);
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          cursor: pointer;
        }

        .remove .glyph {
          width: 12px;
          height: 12px;
        }

        .remove:hover {
          border-color: var(--ia-theme-color-danger-text, #c62828);
          background: color-mix(
            in srgb,
            var(--ia-theme-color-danger-text, #c62828) 10%,
            var(--ia-theme-secondary-background-color, #fff)
          );
          color: var(--ia-theme-color-danger-text, #c62828);
        }
      `]}};_([t({type:Array})],F.prototype,`rows`,void 0),_([t({attribute:!1})],F.prototype,`collapsedIds`,void 0),_([t({type:Object})],F.prototype,`transferStatus`,void 0),_([t({type:Boolean,reflect:!0})],F.prototype,`disabled`,void 0),_([o()],F.prototype,`previewResults`,void 0),F=_([p(`upload-file-tree`)],F);var I=class extends s{constructor(){super(),this.fileList=Ce.empty(),this.skipped=[],this.announcement=``,this.disabled=!1,this.transferStatus={},this.collapseState=fn,this.dragActive=!1,this.scrollOverflows=!1,this.moreAbove=!1,this.moreBelow=!1,this.dropsBeingRead=0,this.foldersPickable=Jt(),this.dragDepth=0,this.observedForResize=[],this.focusAddButtonIfEmptied=!1,this.onDragEnter=e=>{A(e.dataTransfer)&&(e.preventDefault(),this.dragDepth+=1,this.disabled||(this.dragActive=!0))},this.onDragOver=e=>{A(e.dataTransfer)&&(e.preventDefault(),e.dataTransfer&&(e.dataTransfer.dropEffect=this.disabled?`none`:`copy`))},this.onDragLeave=e=>{A(e.dataTransfer)&&(this.dragDepth=Math.max(0,this.dragDepth-1),this.dragDepth===0&&(this.dragActive=!1))},this.onDrop=e=>{let{dataTransfer:t}=e;if(!t||!A(t)||(e.preventDefault(),this.dragDepth=0,this.dragActive=!1,this.disabled))return;let n=an(t);this.startReadingDrop(),n.then(({candidates:e,errors:t})=>{let n=t.length>0;(e.length>0||!n)&&this.dispatchEvent(D(`filesPicked`,{candidates:e})),n&&this.dispatchEvent(D(`filesPickFailed`,{error:t[0]})),this.finishReadingDrop()})},this.measureScrollWindow=()=>{let e=this.scrollWindow;if(!e){this.resetScrollWindow();return}let t=mn(e);this.scrollOverflows=t.overflows,this.moreAbove=t.moreAbove,this.moreBelow=t.moreBelow},this.addEventListener(`dragenter`,this.onDragEnter),this.addEventListener(`dragover`,this.onDragOver),this.addEventListener(`dragleave`,this.onDragLeave),this.addEventListener(`drop`,this.onDrop)}disconnectedCallback(){super.disconnectedCallback(),this.resizeObserver?.disconnect(),this.resizeObserver=void 0,this.observedForResize=[],this.dragDepth=0,this.dragActive=!1}willUpdate(e){e.has(`fileList`)&&(this.collapseState=pn(this.fileList.tree,this.collapseState),this.fileList.totals.fileCount===0&&this.resetScrollWindow()),e.has(`disabled`)&&this.disabled&&(this.dragDepth=0,this.dragActive=!1)}updated(e){this.watchScrollWindow(),e.has(`fileList`)&&this.focusAddButtonIfEmptied&&(this.focusAddButtonIfEmptied=!1,this.fileList.totals.fileCount===0&&this.addMenu?.focusButton())}render(){let e=this.fileList.totals.fileCount>0,t={column:!0,"drag-active":this.dragActive},r=this.isReadingDrops?this.readingText:this.announcement;return i`
      <section
        class=${l(t)}
        aria-label=${a(`Files and folders to upload`)}
      >
        ${this.disabled?n:this.dropZoneTemplate(e)}
        ${this.noticesTemplate}
        <div class="tree-card">
          ${this.treeHeadTemplate}
          ${e?this.scrollWindowTemplate:this.emptyTemplate}
          ${this.footerTemplate}
        </div>
        <div class="sr-only" role="status" aria-live="polite">${r}</div>
        ${this.dragActive?this.dropOverlayTemplate:n}
      </section>
    `}get isReadingDrops(){return this.dropsBeingRead>0}get readingText(){return a(`Reading dropped folders…`)}dropZoneTemplate(e){let t=i`<span class="reading-status">
      <span class="spinner" aria-hidden="true"></span>
      <span>${this.readingText}</span>
    </span>`,n=i`<span class="drop-hint">
      ${a(`or drag and drop them here`)}
    </span>`,r=this.isReadingDrops?t:n;return i`<div class="drop-zone">
      <upload-add-menu
        ?hasFiles=${e}
        ?filesOnly=${!this.foldersPickable}
        @filesPicked=${this.onFilesPicked}
      ></upload-add-menu>
      ${r}
    </div>`}get noticesTemplate(){let e=se(this.skipped);return e.length===0?n:i`<ul class="notices">
      ${e.map(e=>i`<li class="notice">${M}<span>${e}</span></li>`)}
    </ul>`}get treeHeadTemplate(){let e=O(this.fileList.tree),{collapsed:t}=this.collapseState,r=e.some(e=>!t.has(e)),o=e.length===0?n:i`<button
            type="button"
            class="link-button"
            @click=${this.toggleAllFolders}
          >
            ${a(r?`Collapse all`:`Expand all`)}
          </button>`;return i`<div class="tree-head">
      <h2 class="tree-title" id="tree-title">${a(`Files and folders`)}</h2>
      ${o}
    </div>`}get scrollWindowTemplate(){let e={"scroll-frame":!0,"more-above":this.moreAbove,"more-below":this.moreBelow},t=this.scrollOverflows;return i`<div class=${l(e)}>
      <div
        class="tree-scroll"
        tabindex=${f(t?`0`:void 0)}
        role=${f(t?`region`:void 0)}
        aria-labelledby=${f(t?`tree-title`:void 0)}
        @scroll=${this.measureScrollWindow}
      >
        <upload-file-tree
          .rows=${this.fileList.tree}
          .collapsedIds=${this.collapseState.collapsed}
          .transferStatus=${this.transferStatus}
          ?disabled=${this.disabled}
          @folderToggled=${this.onFolderToggled}
          @fileNodeRemoved=${this.onFileNodeRemoved}
        ></upload-file-tree>
      </div>
    </div>`}get emptyTemplate(){return i`<div class="tree-empty">
      <span class="empty-title">${a(`No files yet`)}</span>
      <span
        >${a(`Anything you add appears here, with its folders kept intact.`)}</span
      >
    </div>`}get footerTemplate(){let{totals:e}=this.fileList,t=cn(e),r=hn(t,x(e.fileCount)),a=r?i`<strong>${r[0]}</strong>${r[1]}`:t,o=ln(e.systemFilesSkipped);return i`<div class="tree-foot">
      <span>${a}</span>
      ${o?i`<span>${o}</span>`:n}
    </div>`}get dropOverlayTemplate(){return i`<div class="drop-overlay" aria-hidden="true">
      <span class="drop-overlay-label"
        >${N}${a(`Drop to add`)}</span
      >
    </div>`}onFilesPicked(e){let{candidates:t}=e.detail;this.dispatchEvent(D(`filesPicked`,{candidates:t}))}onFileNodeRemoved(e){this.focusAddButtonIfEmptied=!0;let{nodeId:t}=e.detail;this.dispatchEvent(D(`fileNodeRemoved`,{nodeId:t}))}onFolderToggled(e){let{folderId:t}=e.detail,n=new Set(this.collapseState.collapsed);n.has(t)?n.delete(t):n.add(t),this.collapseState={...this.collapseState,collapsed:n}}toggleAllFolders(){let e=O(this.fileList.tree),{collapsed:t}=this.collapseState,n=e.some(e=>!t.has(e));this.collapseState={...this.collapseState,collapsed:n?new Set(e):new Set}}startReadingDrop(){this.dropsBeingRead+=1,this.dropsBeingRead===1&&this.dispatchEvent(D(`filesReadingChanged`,{reading:!0}))}finishReadingDrop(){this.dropsBeingRead=Math.max(0,this.dropsBeingRead-1),this.dropsBeingRead===0&&this.dispatchEvent(D(`filesReadingChanged`,{reading:!1}))}watchScrollWindow(){let e=[this.scrollWindow,this.treeElement].filter(e=>e instanceof HTMLElement);if(!(e.length===this.observedForResize.length&&e.every((e,t)=>e===this.observedForResize[t])||typeof ResizeObserver>`u`)){this.resizeObserver??(this.resizeObserver=new ResizeObserver(this.measureScrollWindow)),this.resizeObserver.disconnect();for(let t of e)this.resizeObserver.observe(t);this.observedForResize=e}}resetScrollWindow(){this.scrollOverflows=!1,this.moreAbove=!1,this.moreBelow=!1}static get styles(){return[k,Dn,e`
        :host {
          display: block;
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          font-family: var(
            --ia-theme-base-font-family,
            'Helvetica Neue',
            Helvetica,
            Arial,
            sans-serif
          );
          font-size: 14px;
          line-height: 1.43;
        }

        .column {
          position: relative;
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .drop-zone {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          gap: 10px;
          min-height: 120px;
          padding: 20px 16px;
          border: 2px dashed var(--upload-drop-zone-dash);
          border-radius: 6px;
          background: var(--upload-drop-zone-fill);
          text-align: center;
        }

        .drop-hint {
          color: var(--ia-theme-secondary-text-color, #666);
          font-size: 14px;
        }

        /* Nothing to drag from on a touch-only device. */
        @media (hover: none) and (pointer: coarse) {
          .drop-hint {
            display: none;
          }
        }

        .reading-status {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          font-size: 14px;
          font-weight: 700;
        }

        .spinner {
          width: 14px;
          height: 14px;
          flex: none;
          border: 2px solid currentColor;
          border-right-color: transparent;
          border-radius: 50%;
          animation: spin 0.8s linear infinite;
        }

        @keyframes spin {
          to {
            transform: rotate(360deg);
          }
        }

        @media (prefers-reduced-motion: reduce) {
          .spinner {
            animation: none;
          }
        }

        .notices {
          display: flex;
          flex-direction: column;
          gap: 6px;
          margin: 0;
          padding: 0;
          list-style: none;
        }

        .notice {
          display: flex;
          align-items: flex-start;
          gap: 8px;
          padding: 8px 12px;
          border: 1px solid var(--ia-theme-border-color, #ccc);
          border-radius: 3px;
          background: var(--ia-theme-tertiary-background-color, #efefef);
          font-size: 13px;
          overflow-wrap: anywhere;
        }

        .notice .glyph {
          width: 15px;
          height: 15px;
          flex: none;
          margin-top: 1px;
        }

        .tree-card {
          border: 1px solid var(--ia-theme-border-color, #ccc);
          border-radius: 3px;
          background: var(--ia-theme-secondary-background-color, #fff);
        }

        .tree-head {
          display: flex;
          align-items: center;
          justify-content: space-between;
          gap: 12px;
          padding: 8px 12px;
          border-bottom: 1px solid var(--ia-theme-border-color, #ccc);
          font-size: 13px;
        }

        .tree-title {
          margin: 0;
          font-size: 13px;
          font-weight: 700;
          line-height: inherit;
        }

        .link-button {
          padding: 0 1px;
          border: 0;
          border-radius: 2px;
          background: none;
          color: var(--ia-theme-link-color, #4b64ff);
          font: inherit;
          text-decoration: underline;
          text-underline-offset: 2px;
          cursor: pointer;
        }

        .scroll-frame {
          position: relative;
        }

        .tree-scroll {
          overscroll-behavior: contain;
        }

        /* The ring sits inside, where the card's edges can't cover it. */
        .tree-scroll:focus-visible {
          outline-offset: -2px;
        }

        .scroll-frame::before,
        .scroll-frame::after {
          content: '';
          position: absolute;
          z-index: 1;
          left: 0;
          right: 0;
          height: 10px;
          opacity: 0;
          pointer-events: none;
        }

        .scroll-frame::before {
          top: 0;
          background: radial-gradient(
            farthest-side at 50% 0,
            color-mix(
              in srgb,
              var(--ia-theme-primary-text-color, #2c2c2c) 18%,
              transparent
            ),
            transparent
          );
        }

        .scroll-frame::after {
          bottom: 0;
          background: radial-gradient(
            farthest-side at 50% 100%,
            color-mix(
              in srgb,
              var(--ia-theme-primary-text-color, #2c2c2c) 18%,
              transparent
            ),
            transparent
          );
        }

        .scroll-frame.more-above::before,
        .scroll-frame.more-below::after {
          opacity: 1;
        }

        .tree-empty {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 4px;
          padding: 32px 16px;
          color: var(--ia-theme-secondary-text-color, #666);
          font-size: 13px;
          text-align: center;
        }

        .empty-title {
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          font-size: 14px;
          font-weight: 700;
        }

        .tree-foot {
          display: flex;
          flex-wrap: wrap;
          justify-content: space-between;
          gap: 4px 12px;
          padding: 10px 14px;
          border-top: 1px solid var(--ia-theme-border-color, #ccc);
          border-radius: 0 0 2px 2px;
          background: var(--ia-theme-tertiary-background-color, #efefef);
          font-size: 13px;
          font-variant-numeric: tabular-nums;
        }

        .tree-foot strong {
          font-weight: 700;
        }

        .drop-overlay {
          position: absolute;
          z-index: 6;
          inset: 0;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 16px;
          border: 2px solid var(--ia-theme-link-color, #4b64ff);
          border-radius: 6px;
          background: color-mix(
            in srgb,
            var(--upload-drop-zone-fill) 96%,
            transparent
          );
          pointer-events: none;
        }

        .drop-overlay-label {
          /* Centred, but stays in view when a long column is scrolled. */
          position: sticky;
          top: 16px;
          bottom: 16px;
          display: inline-flex;
          align-items: center;
          gap: 8px;
          padding: 8px 14px;
          border-radius: 4px;
          background: var(--ia-theme-secondary-background-color, #fff);
          box-shadow: 0 2px 10px rgba(0, 0, 0, 0.14);
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          font-size: 17px;
          font-weight: 700;
        }

        .drop-overlay-label .glyph {
          width: 18px;
          height: 18px;
          color: var(--ia-theme-link-color, #4b64ff);
        }

        @media (max-width: 767px) {
          .drop-zone {
            min-height: 96px;
            padding: 16px;
          }

          .tree-scroll {
            max-height: 300px;
            overflow-y: auto;
          }
        }
      `]}};_([t({attribute:!1})],I.prototype,`fileList`,void 0),_([t({type:Array})],I.prototype,`skipped`,void 0),_([t({type:String})],I.prototype,`announcement`,void 0),_([t({type:Boolean,reflect:!0})],I.prototype,`disabled`,void 0),_([t({type:Object})],I.prototype,`transferStatus`,void 0),_([o()],I.prototype,`collapseState`,void 0),_([o()],I.prototype,`dragActive`,void 0),_([o()],I.prototype,`scrollOverflows`,void 0),_([o()],I.prototype,`moreAbove`,void 0),_([o()],I.prototype,`moreBelow`,void 0),_([o()],I.prototype,`dropsBeingRead`,void 0),_([m(`upload-add-menu`)],I.prototype,`addMenu`,void 0),_([m(`.tree-scroll`)],I.prototype,`scrollWindow`,void 0),_([m(`upload-file-tree`)],I.prototype,`treeElement`,void 0),I=_([p(`upload-file-column`)],I);function Xn(e){switch(e){case`movies`:return a(`Community movies`);case`audio`:return a(`Community audio`);case`texts`:return a(`Community text`);case`software`:return a(`Community software`);case`image`:return a(`Community image`);case`data`:return a(`Community data`);default:return}}function Zn(e){let t=e.map(e=>({value:`${e.mediatype}:${e.collection}`,label:Xn(e.mediatype)??e.collection}));return[{value:``,label:a(`Choose a collection`)},...t]}var Qn=10;function $n(e){let t=e.trim();return/^\d{4}(?:-\d{2}){0,2}$/.test(t)||t.length>=Qn}var L=e`
  *,
  *::before,
  *::after {
    box-sizing: border-box;
  }

  [hidden] {
    display: none !important;
  }

  :focus-visible {
    outline: 2px solid var(--ia-theme-link-color, #4b64ff);
    outline-offset: 2px;
  }

  .field {
    display: grid;
    gap: 6px;
    min-width: 0;
  }

  .label-row {
    display: flex;
    align-items: center;
    gap: 4px 10px;
    flex-wrap: wrap;
  }

  .label {
    font-size: 14px;
    font-weight: 700;
  }

  .state {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 13px;
    font-weight: 700;
  }

  .glyph {
    width: 15px;
    height: 15px;
    flex: none;
  }

  .glyph-cutout {
    stroke: var(--ia-theme-secondary-background-color, #fff);
  }

  .glyph-cutout-fill {
    fill: var(--ia-theme-secondary-background-color, #fff);
  }

  .state.satisfied {
    color: var(--ia-theme-color-success-text, #267326);
  }

  .state.missing {
    color: var(--ia-theme-color-danger-text, #c62828);
  }

  .state.neutral {
    color: var(--ia-theme-primary-text-color, #333);
    font-weight: 400;
  }

  .state.optional {
    color: var(--ia-theme-secondary-text-color, #595959);
    font-weight: 400;
  }

  .control {
    display: block;
    width: 100%;
    height: 38px;
    margin: 0;
    padding: 0 10px;
    border: 1px solid var(--ia-theme-field-border-color, #767676);
    border-radius: 3px;
    background: var(--ia-theme-secondary-background-color, #fff);
    color: var(--ia-theme-primary-text-color, #333);
    font: inherit;
    font-size: 15px;
  }

  .control::placeholder {
    color: var(--ia-theme-secondary-text-color, #6b6b6b);
    opacity: 1;
  }

  .control.danger {
    border: 2px solid var(--ia-theme-color-danger-text, #c62828);
    padding-inline: 9px;
  }

  /* Locked while uploading: muted text, a dashed border and no text cursor,
     so it doesn't rest on colour alone. The form also says so in words. */
  .control:read-only:not(select),
  select.control:disabled {
    border-style: dashed;
    color: var(--ia-theme-secondary-text-color, #595959);
    cursor: default;
  }

  /* A select with its own arrow, drawn in the text colour so it follows
     the theme. */
  .select-wrap {
    position: relative;
  }

  .select-wrap select.control {
    appearance: none;
    -webkit-appearance: none;
    padding-right: 34px;
    cursor: pointer;
  }

  .select-wrap select.control:disabled {
    cursor: default;
  }

  .select-wrap::after {
    content: '';
    position: absolute;
    top: 50%;
    right: 12px;
    width: 12px;
    height: 8px;
    margin-top: -4px;
    background: var(--ia-theme-primary-text-color, #333);
    -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 12 8'%3E%3Cpath d='M1 1.5l5 5 5-5' fill='none' stroke='%23000' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E")
      no-repeat center / contain;
    mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 12 8'%3E%3Cpath d='M1 1.5l5 5 5-5' fill='none' stroke='%23000' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E")
      no-repeat center / contain;
    pointer-events: none;
  }

  .select-wrap:has(select:disabled)::after {
    background: var(--ia-theme-secondary-text-color, #595959);
  }

  .help {
    margin: 0;
    font-size: 13px;
    color: var(--ia-theme-secondary-text-color, #595959);
  }

  .error {
    display: flex;
    align-items: flex-start;
    gap: 5px;
    margin: 0;
    font-size: 13px;
    font-weight: 700;
    color: var(--ia-theme-color-danger-text, #c62828);
  }

  .error .glyph {
    margin-top: 1px;
  }

  /* The white outline button of the file column, in both themes: #194880
     text needs a light surface to meet contrast. */
  .button-secondary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 34px;
    padding: 0 12px;
    border: 1px solid var(--ia-theme-primary-cta-fill, #194880);
    border-radius: 4px;
    background: #fff;
    color: var(--ia-theme-primary-cta-fill, #194880);
    font: inherit;
    font-size: 14px;
    font-weight: 700;
    line-height: 1;
    white-space: nowrap;
    cursor: pointer;
  }

  .button-secondary:hover {
    background: #edf1ff;
  }

  .button-secondary:disabled {
    cursor: default;
    opacity: 0.6;
  }

  .button-secondary .icon {
    width: 14px;
    height: 14px;
    flex: none;
  }

  .link-button {
    padding: 0 1px;
    border: 0;
    border-radius: 2px;
    background: none;
    color: var(--ia-theme-link-color, #4b64ff);
    font: inherit;
    font-weight: 700;
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: -1px;
    padding: 0;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  /* 16px text stops iOS Safari zooming in on focus. */
  @media (max-width: 767px) {
    .control {
      font-size: 16px;
    }
  }
`,er=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <circle cx="8" cy="8" r="7.4" fill="currentColor" />
  <path
    class="glyph-cutout"
    d="M4.5 8.3l2.3 2.3 4.7-4.9"
    fill="none"
    stroke-width="1.9"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>`,R=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M8 1.3 15.3 14.4H.7Z"
    fill="currentColor"
    stroke="currentColor"
    stroke-width=".8"
    stroke-linejoin="round"
  />
  <path
    class="glyph-cutout"
    d="M8 6.1v3.7"
    stroke-width="1.8"
    stroke-linecap="round"
  />
  <circle class="glyph-cutout-fill" cx="8" cy="12" r="1" />
</svg>`,tr=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <circle
    cx="8"
    cy="8"
    r="6.6"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
  />
  <circle cx="8" cy="8" r="2" fill="currentColor" />
</svg>`,nr=i`<svg
  class="glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M5 7V5a3 3 0 0 1 6 0v2"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
  <rect x="2.8" y="7" width="10.4" height="7.6" rx="1.4" fill="currentColor" />
  <circle class="glyph-cutout-fill" cx="8" cy="10.8" r="1.2" />
</svg>`,rr=i`<svg
  class="disclosure"
  viewBox="0 0 10 10"
  aria-hidden="true"
  focusable="false"
>
  <path d="M2 .6 9.2 5 2 9.4Z" fill="currentColor" />
</svg>`,ir=i`<svg
  class="icon"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M8 2.5v11M2.5 8h11"
    stroke="currentColor"
    stroke-width="2.2"
    stroke-linecap="round"
  />
</svg>`,ar=i`<svg
  class="icon"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M3.5 3.5l9 9M12.5 3.5l-9 9"
    stroke="currentColor"
    stroke-width="2.2"
    stroke-linecap="round"
  />
</svg>`,or=i`<svg
  class="icon"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M6.5 4h8M6.5 8h8M6.5 12h8"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
  <path
    d="M2.2 2.6h1v3M1.8 5.6h1.8M1.8 8.1c.3-.5 1.8-.6 1.8.3 0 .7-1.8 1.1-1.8 1.9h1.9M1.8 11.6c.4-.4 1.8-.4 1.8.4 0 .5-.5.6-.9.6.5 0 1 .2 1 .7 0 .8-1.5.9-1.9.4"
    fill="none"
    stroke="currentColor"
    stroke-width=".9"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>`,sr=i`<svg
  class="icon"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M6.5 4h8M6.5 8h8M6.5 12h8"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
  <circle cx="2.6" cy="4" r="1.3" fill="currentColor" />
  <circle cx="2.6" cy="8" r="1.3" fill="currentColor" />
  <circle cx="2.6" cy="12" r="1.3" fill="currentColor" />
</svg>`,cr=i`<svg
  class="icon"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M6.8 9.2a3 3 0 0 0 4.2 0l2.3-2.3a3 3 0 0 0-4.2-4.2l-1 1"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
  <path
    d="M9.2 6.8a3 3 0 0 0-4.2 0L2.7 9.1a3 3 0 0 0 4.2 4.2l1-1"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
</svg>`,lr=i`<svg
  class="icon"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M11.4 8.8l1.9-1.9a3 3 0 0 0-4.2-4.2L7.2 4.6M4.6 7.2l-1.9 1.9a3 3 0 0 0 4.2 4.2l1.9-1.9"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
  <path
    d="M5.2 1.4v1.8M1.4 5.2h1.8M10.8 12.8v1.8M12.8 10.8h1.8"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
</svg>`,ur=i`<svg
  class="icon"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M2.2 3h8.6M6.5 3v10.2"
    fill="none"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
  <path
    d="M10.2 10l3.6 3.6M13.8 10l-3.6 3.6"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
</svg>`,dr=i`<svg
  class="icon"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M3 8.4l3.2 3.2L13 4.6"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>`,fr=i`<svg
  class="icon"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M3.5 6l4.5 4.5L12.5 6"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
  />
</svg>`;function pr(e){let t=e=>({value:e.code,label:e.name}),n=[{groupLabel:a(`Common languages`),options:e.filter(e=>e.top).map(t)},{groupLabel:a(`All languages`),options:e.filter(e=>!e.top).map(t)}];return[{value:``,label:a(`Choose a language`)},...n.filter(e=>e.options.length>0)]}function z(e){switch(e){case`title`:return a(`Item Title`);case`identifier`:return a(`Page URL`);case`description`:return a(`Description`);case`subjects`:return a(`Subject Tags`);case`collection`:return a(`Collection`)}}function mr(e,t){let n=w.length;if(!e)return a(r`${n} fields`);let i=w.filter(e=>t[e]!==`satisfied`).length;return a(r`${i} of ${n} still needed`)}function hr(e){switch(e){case`date`:return`recommended`;case`custom`:return`additional`;default:return`required`}}function gr(e){return e.collections.filter(e=>!_r(e)).map(e=>{let t=e.title.trim()!==``;return{value:e.value,title:t?e.title:e.identifier,identifier:e.identifier,hasTitle:t}})}function _r(e){let[,t=``]=e.value.split(`:`);return[e.identifier,t].some(e=>e.toLowerCase()===ke)}function vr(e){let t=e.trim().length;return t===0?`off`:t<3?`tooShort`:`searching`}function yr(){return a(r`Type at least ${3} characters to search all the collections you can upload to.`)}function br(e){return a(r`No community collection matches "${e.trim()}".`)}function xr(e){return a(r`No collections you can upload to match "${e.trim()}".`)}function Sr(e){let{term:t,communityCount:n,searchState:i,resultCount:o}=e;if(i===`done`)return wr(t,o);let s=t.trim()!==``,c;c=s?n===0?br(t):a(n===1?`1 community collection matches.`:r`${n} community collections match.`):a(n===1?`1 community collection.`:r`${n} community collections.`);let l=Cr(i);return l===``?c:`${c} ${l}`}function Cr(e){switch(e){case`off`:return``;case`tooShort`:return yr();case`searching`:return a(`Searching…`);case`failed`:return a(`Couldn't search collections.`)}}function wr(e,t){return t===0?xr(e):a(t===1?`1 collection you can upload to matches.`:r`${t} collections you can upload to match.`)}function Tr(e,t){if(e===``)return{text:``,identifier:``};let n=t.community.find(t=>t.value===e);if(n){let e=n.hasTitle?n.identifier:``;return{text:n.title,identifier:e}}let r=e.indexOf(`:`),i=r===-1?e:e.slice(r+1),a=t.titles.get(e);return a?{text:a,identifier:i}:{text:i,identifier:``}}function Er(e){return e.map(({mediatype:e,collection:t})=>{let n=Xn(e);return{value:`${e}:${t}`,title:n??t,identifier:t,hasTitle:n!==void 0}})}function Dr(e,t){let n=e.trim().toLowerCase();if(n===``)return;let r=e=>e.identifier.toLowerCase()===n||e.value.toLowerCase()===n;return t.community.find(e=>r(e)||e.title.toLowerCase()===n)??t.results.find(r)}function B(e){let{id:t,state:n}=e;switch(n){case`neutral`:return i`<span class="state neutral" id=${t}
        >${tr}${a(`Required`)}</span
      >`;case`satisfied`:return i`<span class="state satisfied" id=${t}
        >${er}${e.satisfiedText??a(`filled`)}</span
      >`;case`missing`:return i`<span class="state missing" id=${t}
        >${R}${a(`Required — not yet filled`)}</span
      >`;case`optional`:return i`<span class="state optional" id=${t}
        >${a(`Optional`)}</span
      >`}}function Or(e,t){let n=t.trim().toLowerCase();return n===``?e:e.filter(e=>e.title.toLowerCase().includes(n)||e.identifier.toLowerCase().includes(n))}function kr(e){return`option-${encodeURIComponent(e)}`}function Ar(e,t,n){let r=new Map(n.map(e=>[e.value,e])),i=t.map(e=>r.get(e.value)??e),a=new Set(i.map(e=>e.value)),o=Or(n,e).filter(e=>!a.has(e.value)),s=e.trim().toLowerCase(),c=e=>e.title.toLowerCase().startsWith(s)||e.identifier.toLowerCase().startsWith(s)?0:1;return[...i,...o].sort((e,t)=>c(e)-c(t))}var V=class extends s{constructor(...e){super(...e),this.value=``,this.fieldState=`neutral`,this.help=``,this.openCollections=[],this.suggestedCollection=``,this.disabled=!1,this.expanded=!1,this.searchState=`off`,this.results=[],this.community=[],this.titles=new Map,this.lastValidChoice=``,this.searchedTerm=``,this.searchGeneration=0,this.revealActive=!1}focusControl(){this.input?.focus()}disconnectedCallback(){super.disconnectedCallback(),this.cancelSearch()}willUpdate(e){if(e.has(`openCollections`)&&(this.community=Er(this.openCollections)),e.has(`value`)){let e=this.value===this.sentValue;this.sentValue=void 0,this.value!==``&&(this.lastValidChoice=this.value),e||(this.unmatchedText=void 0,this.inputFocused||(this.query=void 0))}e.has(`disabled`)&&this.disabled&&this.close()}updated(){this.revealActive&&(this.revealActive=!1,this.activeId&&this.shadowRoot?.getElementById(this.activeId)?.scrollIntoView({block:`nearest`}))}render(){let e=this.display(this.value),t=this.query===void 0,r=this.query??e.text,o=t&&e.identifier!==``,s=this.unmatchedText!==void 0,c=s||this.fieldState===`missing`,d=this.helpText,p=[o?`value-identifier`:``,`state`,d===``?``:`help`].filter(e=>e!==``).join(` `),m=this.listModel,h=this.activeItemIn(m),g=this.expanded?h?.id:void 0,_=r!==``&&!this.disabled;return i`
      <div class="field">
        <div class="label-row">
          <label class="label" for="control">${a(`Collection`)}</label>
          ${s?i`<span class="state missing" id="state"
                >${R}${a(`Pick a collection from the list`)}</span
              >`:B({id:`state`,state:this.fieldState})}
        </div>
        <div class="combo">
          <div class="input-box">
            <input
              id="control"
              class=${l({control:!0,"combo-input":!0,danger:c})}
              type="text"
              role="combobox"
              autocomplete="off"
              autocapitalize="none"
              spellcheck="false"
              aria-autocomplete="list"
              aria-expanded=${this.expanded?`true`:`false`}
              aria-controls="listbox"
              aria-activedescendant=${f(g)}
              aria-required="true"
              aria-invalid=${f(c?`true`:void 0)}
              aria-describedby=${p}
              placeholder=${a(`Choose a collection, or type to search`)}
              .value=${u(r)}
              ?readonly=${this.disabled}
              @focus=${this.handleFocus}
              @click=${this.handleFocus}
              @input=${this.handleInput}
              @keydown=${this.handleKeydown}
              @blur=${this.handleBlur}
            />
            <span
              class="value-overlay"
              aria-hidden="true"
              ?hidden=${!o}
            >
              <span class="overlay-text">${r}</span
              ><span class="overlay-separator"></span
              ><span class="overlay-identifier" id="value-identifier"
                >${e.identifier}</span
              >
            </span>
            <span class="combo-buttons">
              ${_?i`<button
                    type="button"
                    class="combo-button clear"
                    tabindex="-1"
                    aria-label=${a(`Clear the collection`)}
                    title=${a(`Clear the collection`)}
                    @mousedown=${this.keepInputFocus}
                    @click=${this.handleClear}
                  >
                    ${ar}
                  </button>`:n}
              <button
                type="button"
                class="combo-button chevron"
                tabindex="-1"
                aria-label=${a(`Collections`)}
                aria-expanded=${this.expanded?`true`:`false`}
                aria-controls="listbox"
                ?disabled=${this.disabled}
                @mousedown=${this.keepInputFocus}
                @click=${this.handleChevron}
              >
                ${fr}
              </button>
            </span>
          </div>
          ${this.popupTemplate(m,g)}
        </div>
        ${d===``?n:i`<p class="help" id="help">${d}</p>`}
        <span class="sr-only" role="status">${this.statusText(m)}</span>
      </div>
    `}popupTemplate(e,t){let{term:r,options:o}=e,{searchState:s}=this,c=s===`tooShort`&&o.length===0,l=s===`done`&&o.length===0;return i`
      <div class="popup" ?hidden=${!this.expanded}>
        <div id="listbox" role="listbox" aria-label=${a(`Collections`)}>
          ${o.map(e=>this.optionTemplate(e,kr(e.value),t))}
          ${c?i`<div class="list-note" aria-hidden="true">
                ${br(r)}
              </div>`:n}
          ${l?i`<div class="list-note" aria-hidden="true">
                ${xr(r)}
              </div>`:n}
          ${s===`searching`?i`<div class="list-note" aria-hidden="true">
                ${a(`Searching…`)}
              </div>`:n}
          ${s===`failed`?this.retryTemplate(t):n}
        </div>
        ${s===`tooShort`?i`<p class="popup-hint" aria-hidden="true">
              ${yr()}
            </p>`:n}
      </div>
    `}retryTemplate(e){return i`
      <div class="list-note failed" aria-hidden="true">
        ${R}${a(`Couldn't search collections.`)}
      </div>
      <div
        id="retry-option"
        class=${l({option:!0,retry:!0,active:e===`retry-option`})}
        role="option"
        aria-selected="false"
        @mousedown=${this.keepInputFocus}
        @pointermove=${()=>this.setActive(`retry-option`)}
        @click=${this.retrySearch}
      >
        <span class="check"></span>
        <span class="option-text">
          <span class="retry-label">${a(`Try again`)}</span>
        </span>
      </div>
    `}optionTemplate(e,t,r){let o=e.value===this.value,s=this.suggestedCollection!==``&&e.value===this.suggestedCollection;return i`
      <div
        id=${t}
        class=${l({option:!0,active:t===r,selected:o})}
        role="option"
        aria-selected=${o?`true`:`false`}
        @mousedown=${this.keepInputFocus}
        @pointermove=${()=>this.setActive(t)}
        @click=${()=>this.choose(e)}
      >
        <span class="check">${o?dr:n}</span>
        <span class="option-text">
          <span class="option-label">${e.title}</span>
          ${e.hasTitle?i`<span class="option-identifier">${e.identifier}</span>`:n}
          ${s?i`<span class="option-suggested"
                >${a(`suggested for your files`)}</span
              >`:n}
        </span>
      </div>
    `}get helpText(){if(this.unmatchedText===void 0)return this.help;let e=[xr(this.unmatchedText),a(`Check the spelling, or pick a Community collection.`)];return this.lastValidChoice!==``&&e.push(a(`Esc brings back the previous choice.`)),e.join(` `)}statusText(e){if(!this.expanded)return this.unmatchedText===void 0?``:a(`Pick a collection from the list`);let t=e.options.length;return Sr({term:e.term,communityCount:t,searchState:this.searchState,resultCount:t})}get listModel(){let e=(this.query??``).trim(),t=this.searchState===`done`?Ar(e,this.results,this.community):Or(this.community,e),n=t.map(e=>({kind:`choice`,id:kr(e.value),option:e}));return this.searchState===`failed`&&n.push({kind:`retry`,id:`retry-option`}),{term:e,options:t,items:n}}activeItemIn(e){return e.items.find(e=>e.id===this.activeId)}get inputFocused(){let{input:e}=this;return e!==void 0&&this.shadowRoot?.activeElement===e}display(e){return Tr(e,{community:this.community,titles:this.titles})}handleFocus(){!this.disabled&&!this.expanded&&this.open()}handleInput(e){!(e.target instanceof HTMLInputElement)||this.disabled||(this.query=e.target.value,this.expanded=!0,this.activeId=void 0,this.searchFor(this.query.trim()))}handleKeydown(e){if(!this.disabled)switch(e.key){case`ArrowDown`:e.preventDefault(),this.expanded||this.open(),e.altKey||this.moveActive(1);return;case`ArrowUp`:if(e.preventDefault(),e.altKey){this.close();return}this.expanded||this.open(),this.moveActive(-1);return;case`Home`:case`End`:{let{items:t}=this.listModel;if(!(this.expanded&&this.activeId!==void 0)||t.length===0)return;e.preventDefault();let n=e.key===`Home`?t[0]:t[t.length-1];this.setActive(n.id,!0);return}case`Enter`:this.handleEnter(e);return;case`Escape`:this.handleEscape(e);return;default:return}}handleEnter(e){let t=this.expanded?this.activeItemIn(this.listModel):void 0;if(t){e.preventDefault(),t.kind===`retry`?this.retrySearch():this.choose(t.option);return}let n=this.exactMatch();n&&(e.preventDefault(),this.choose(n))}handleEscape(e){if(this.expanded){e.preventDefault(),this.close();return}(this.query!==void 0||this.value!==this.lastValidChoice)&&(e.preventDefault(),this.restoreLastChoice())}handleBlur(){if(this.close(),this.query===void 0)return;let e=this.query.trim();if(e===``){this.showChoice(),this.value!==``&&this.emitValue(``);return}let t=this.exactMatch();if(t){this.choose(t);return}if(this.value!==``&&e.toLowerCase()===this.display(this.value).text.toLowerCase()){this.showChoice();return}this.unmatchedText=e,this.value!==``&&this.emitValue(``)}handleClear(){this.query=``,this.unmatchedText=void 0,this.searchFor(``),this.value!==``&&this.emitValue(``),this.input?.focus(),this.expanded||this.open()}handleChevron(){if(!this.disabled){if(this.expanded){this.close();return}this.open(),this.input?.focus()}}keepInputFocus(e){e.preventDefault()}open(){this.expanded=!0}close(){this.expanded=!1,this.activeId=void 0}moveActive(e){let{items:t,term:n}=this.listModel;if(t.length===0)return;let r=t.findIndex(e=>e.id===this.activeId);if(r!==-1){let n=(r+e+t.length)%t.length;this.setActive(t[n].id,!0);return}let i=n===``?t.find(e=>e.kind===`choice`&&e.option.value===this.value):void 0,a=e===1?t[0]:t[t.length-1];this.setActive((i??a).id,!0)}setActive(e,t=!1){this.activeId=e,t&&(this.revealActive=!0)}exactMatch(){if(this.query!==void 0)return Dr(this.query,{community:this.community,results:this.searchState===`done`?this.results:[]})}choose(e){e.hasTitle&&this.titles.set(e.value,e.title),this.close(),this.showChoice(),e.value!==this.value&&this.emitValue(e.value)}restoreLastChoice(){this.showChoice(),this.value!==this.lastValidChoice&&this.emitValue(this.lastValidChoice)}showChoice(){this.query=void 0,this.unmatchedText=void 0,this.searchFor(``)}searchFor(e){if(e===this.searchedTerm||(this.cancelSearch(),this.searchedTerm=e,this.results=[],this.searchState=vr(e),this.searchState!==`searching`))return;let t=this.searchGeneration;this.debounceTimer=setTimeout(()=>{this.debounceTimer=void 0,this.search(e,t)},600)}retrySearch(){let e=this.searchedTerm;this.cancelSearch(),this.activeId=void 0,this.searchState=`searching`,this.search(e,this.searchGeneration)}async search(e,t){let{searchCollections:n}=this,r;try{if(!n)throw Error(`No collection search`);r=await n(e)}catch{if(t!==this.searchGeneration)return;this.searchState=`failed`;return}if(t!==this.searchGeneration)return;let i=gr(r);for(let e of i)e.hasTitle&&this.titles.set(e.value,e.title);this.results=i,this.searchState=`done`,this.activeId!==void 0&&(this.revealActive=!0),this.chooseLateMatch(e)}chooseLateMatch(e){if(this.unmatchedText!==e||this.inputFocused)return;let t=this.exactMatch();t&&this.choose(t)}cancelSearch(){clearTimeout(this.debounceTimer),this.debounceTimer=void 0,this.searchGeneration+=1}emitValue(e){this.sentValue=e,this.dispatchEvent(D(`metadataChanged`,{changes:{collection:e}}))}static get styles(){return[L,e`
      :host {
        --upload-combo-buttons-width: 64px;
      }

      .combo {
        position: relative;
      }

      .input-box {
        position: relative;
      }

      .control.combo-input {
        padding-right: var(--upload-combo-buttons-width);
        text-overflow: ellipsis;
      }

      .control.combo-input.danger {
        padding-right: calc(var(--upload-combo-buttons-width) - 1px);
      }

      /* Lines the identifier up after the input's own text: an invisible
         copy of the text, at the input's text inset (border and padding). */
      .value-overlay {
        position: absolute;
        inset: 0 var(--upload-combo-buttons-width) 0 11px;
        display: flex;
        align-items: center;
        overflow: hidden;
        font-size: 15px;
        white-space: pre;
        pointer-events: none;
      }

      .overlay-text {
        flex: none;
        visibility: hidden;
      }

      .overlay-separator::before {
        content: '\\00b7';
        margin: 0 6px;
        color: var(--ia-theme-secondary-text-color, #595959);
      }

      .overlay-identifier {
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        color: var(--ia-theme-secondary-text-color, #595959);
        font-size: 13px;
      }

      .combo-buttons {
        position: absolute;
        top: 0;
        right: 4px;
        bottom: 0;
        display: flex;
        align-items: center;
        gap: 2px;
      }

      .combo-button {
        display: grid;
        place-items: center;
        width: 28px;
        height: 28px;
        padding: 0;
        border: 0;
        border-radius: 3px;
        background: none;
        color: var(--ia-theme-primary-text-color, #333);
        cursor: pointer;
      }

      .combo-button:hover:not(:disabled) {
        background: var(--ia-theme-tertiary-background-color, #efefef);
      }

      .combo-button:disabled {
        color: var(--ia-theme-secondary-text-color, #595959);
        cursor: default;
      }

      .combo-button.clear .icon {
        width: 11px;
        height: 11px;
      }

      .combo-button.chevron .icon {
        width: 16px;
        height: 16px;
        transition: transform 0.15s ease;
      }

      .combo-button.chevron[aria-expanded='true'] .icon {
        transform: rotate(180deg);
      }

      .popup {
        position: absolute;
        z-index: 10;
        top: calc(100% + 4px);
        left: 0;
        right: 0;
        max-height: 320px;
        padding: 4px;
        overflow-y: auto;
        overscroll-behavior: contain;
        border: 1px solid var(--ia-theme-field-border-color, #767676);
        border-radius: 3px;
        background: var(--ia-theme-secondary-background-color, #fff);
        color: var(--ia-theme-primary-text-color, #333);
        box-shadow: 0 8px 22px rgba(0, 0, 0, 0.16);
      }

      .option {
        display: flex;
        align-items: flex-start;
        gap: 6px;
        padding: 6px 10px 6px 6px;
        border-radius: 3px;
        line-height: 1.35;
        cursor: pointer;
      }

      .option.active {
        background: color-mix(
          in srgb,
          var(--ia-theme-link-color, #4b64ff) 16%,
          var(--ia-theme-secondary-background-color, #fff)
        );
        outline: 1px solid var(--ia-theme-link-color, #4b64ff);
      }

      .check {
        display: grid;
        place-items: center;
        flex: none;
        width: 18px;
        height: 1.35em;
        color: var(--ia-theme-link-color, #4b64ff);
      }

      .check .icon {
        width: 14px;
        height: 14px;
      }

      .option-text {
        display: flex;
        flex-wrap: wrap;
        align-items: baseline;
        gap: 2px 8px;
        min-width: 0;
      }

      .option-label {
        overflow-wrap: anywhere;
      }

      .option.selected .option-label {
        font-weight: 700;
      }

      .option-identifier {
        color: var(--ia-theme-secondary-text-color, #595959);
        font-size: 13px;
        overflow-wrap: anywhere;
      }

      .option-suggested {
        padding: 0 6px;
        border: 1px solid var(--ia-theme-border-color, #ccc);
        border-radius: 9px;
        color: var(--ia-theme-secondary-text-color, #595959);
        font-size: 12px;
        white-space: nowrap;
      }

      .retry-label {
        color: var(--ia-theme-link-color, #4b64ff);
        font-weight: 700;
        text-decoration: underline;
        text-underline-offset: 2px;
      }

      .list-note {
        display: flex;
        align-items: center;
        gap: 5px;
        padding: 4px 10px 6px 30px;
        color: var(--ia-theme-secondary-text-color, #595959);
        font-size: 13px;
      }

      .list-note.failed {
        color: var(--ia-theme-color-danger-text, #c62828);
        font-weight: 700;
      }

      .popup-hint {
        margin: 4px 0 0;
        padding: 8px 10px 4px;
        border-top: 1px solid var(--ia-theme-border-color, #ccc);
        color: var(--ia-theme-secondary-text-color, #595959);
        font-size: 13px;
      }

      @media (max-width: 767px) {
        .value-overlay {
          font-size: 16px;
        }
      }

      @media (prefers-reduced-motion: reduce) {
        .combo-button.chevron .icon {
          transition: none;
        }
      }
    `]}};_([t({type:String})],V.prototype,`value`,void 0),_([t({type:String})],V.prototype,`fieldState`,void 0),_([t({type:String})],V.prototype,`help`,void 0),_([t({attribute:!1})],V.prototype,`openCollections`,void 0),_([t({type:String})],V.prototype,`suggestedCollection`,void 0),_([t({attribute:!1})],V.prototype,`searchCollections`,void 0),_([t({type:Boolean})],V.prototype,`disabled`,void 0),_([o()],V.prototype,`query`,void 0),_([o()],V.prototype,`expanded`,void 0),_([o()],V.prototype,`activeId`,void 0),_([o()],V.prototype,`searchState`,void 0),_([o()],V.prototype,`results`,void 0),_([o()],V.prototype,`unmatchedText`,void 0),_([m(`#control`)],V.prototype,`input`,void 0),V=_([p(`upload-collection-combobox`)],V);var H=class extends s{constructor(...e){super(...e),this.rows=[],this.errors={},this.disabled=!1,this.untouchedRowIds=new Set,this.rowCounter=0,this.latestRows=[],this.unconfirmedRows=[]}async focusControl(){this.untouchedRowIds.size>0&&(this.untouchedRowIds=new Set,await this.updateComplete);let e=this.rows.find(e=>this.errors[e.id]);if(e){let t=this.errors[e.id]?.key!==void 0;this.focusTarget({rowId:e.id,part:t?`key`:`value`});return}let[t]=this.rows;this.focusTarget(t?{rowId:t.id,part:`key`}:`add`)}willUpdate(e){if(!e.has(`rows`)||this.untouchedRowIds.size===0)return;let t=new Set(this.rows.map(e=>e.id)),n=[...this.untouchedRowIds].filter(e=>t.has(e));n.length<this.untouchedRowIds.size&&(this.untouchedRowIds=new Set(n))}updated(){this.pendingFocus&&this.focusTarget(this.pendingFocus)&&(this.pendingFocus=void 0)}render(){return i`
      ${this.rows.length>0?i`<ul class="rows" aria-label=${a(`Custom metadata`)}>
            ${d(this.rows,e=>e.id,(e,t)=>this.rowTemplate(e,t))}
          </ul>`:n}
      <div>
        <button
          type="button"
          id="add"
          class="button-secondary"
          ?disabled=${this.disabled}
          @click=${this.addRow}
        >
          ${ir}${a(`Add custom metadata`)}
        </button>
      </div>
    `}rowTemplate(e,t){let o=this.untouchedRowIds.has(e.id)?void 0:this.errors[e.id],s=o?.key,c=o?.value!==void 0,d=`key-${t}`,p=`value-${t}`,m=`key-error-${t}`,h=`value-error-${t}`,g=t+1,_=e.key.trim(),v=a(_===``?r`Remove custom metadata row ${g}`:r`Remove custom metadata row ${g} (${_})`);return i`
      <li
        class="row"
        @focusout=${t=>this.handleRowFocusOut(e.id,t)}
      >
        <label class="small-label key-label" for=${d}>${a(`Key`)}</label>
        <input
          id=${d}
          aria-label=${a(r`Key, row ${g}`)}
          class=${l({control:!0,"key-input":!0,danger:s!==void 0})}
          type="text"
          spellcheck="false"
          autocapitalize="none"
          autocomplete="off"
          .value=${u(e.key)}
          aria-invalid=${f(s?`true`:void 0)}
          aria-describedby=${f(s?m:void 0)}
          ?readonly=${this.disabled}
          @input=${t=>this.editRow(e.id,`key`,t)}
        />
        ${s?i`<p class="error key-error" id=${m}>
              ${R}${it(s,e.key)}
            </p>`:n}
        <label class="small-label value-label" for=${p}
          >${a(`Value`)}</label
        >
        <input
          id=${p}
          aria-label=${a(r`Value, row ${g}`)}
          class=${l({control:!0,"value-input":!0,danger:c})}
          type="text"
          .value=${u(e.value)}
          aria-invalid=${f(c?`true`:void 0)}
          aria-describedby=${f(c?h:void 0)}
          ?readonly=${this.disabled}
          @input=${t=>this.editRow(e.id,`value`,t)}
        />
        ${c?i`<p class="error value-error" id=${h}>
              ${R}${at()}
            </p>`:n}
        <button
          type="button"
          class="remove"
          aria-label=${v}
          ?disabled=${this.disabled}
          @click=${()=>this.removeRow(e.id)}
        >
          ${ar}
        </button>
      </li>
    `}addRow(){let e={id:this.newRowId(),key:``,value:``};this.pendingFocus={rowId:e.id,part:`key`},this.untouchedRowIds=new Set([...this.untouchedRowIds,e.id]),this.emitRows([...this.currentRows(),e])}handleRowFocusOut(e,t){let{currentTarget:n,relatedTarget:r}=t;n instanceof Element&&r instanceof Node&&n.contains(r)||!this.untouchedRowIds.has(e)||(this.untouchedRowIds=new Set([...this.untouchedRowIds].filter(t=>t!==e)))}editRow(e,t,n){if(!(n.target instanceof HTMLInputElement))return;let r=n.target.value,i=this.currentRows().map(n=>n.id===e?{...n,[t]:r}:n);this.emitRows(i)}removeRow(e){let t=this.currentRows(),n=t.findIndex(t=>t.id===e),r=t[n+1]??t[n-1];this.pendingFocus=r?{rowId:r.id,part:`key`}:`add`,this.emitRows(t.filter(t=>t.id!==e))}emitRows(e){this.latestRows=e,this.unconfirmedRows=[...this.unconfirmedRows,e],this.dispatchEvent(D(`metadataChanged`,{changes:{custom:e}}))}currentRows(){if(this.rows===this.reconciledRows)return this.latestRows;this.reconciledRows=this.rows;let e=this.unconfirmedRows.indexOf(this.rows);return e>=0?this.unconfirmedRows=this.unconfirmedRows.slice(e+1):(this.unconfirmedRows=[],this.latestRows=this.rows),this.latestRows}newRowId(){let e=this.currentRows(),t;do this.rowCounter+=1,t=`custom-${this.rowCounter}`;while(e.some(e=>e.id===t));return t}focusTarget(e){if(e===`add`)return this.addButton?.focus(),this.addButton!==void 0;let t=this.rows.findIndex(t=>t.id===e.rowId);if(t<0)return!1;let n=this.shadowRoot?.getElementById(`${e.part}-${t}`);return n?.focus(),n!=null}static get styles(){return[L,e`
      :host {
        display: grid;
        gap: 12px;
      }

      .rows {
        display: grid;
        gap: 12px;
        margin: 0;
        padding: 0;
        list-style: none;
      }

      .row {
        display: grid;
        grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) 28px;
        grid-template-areas:
          'key-label value-label .'
          'key-input value-input remove'
          'key-error value-error .';
        gap: 4px 10px;
        align-items: center;
      }

      .small-label {
        font-size: 13px;
        font-weight: 700;
      }

      .key-label {
        grid-area: key-label;
      }

      .value-label {
        grid-area: value-label;
      }

      .key-input {
        grid-area: key-input;
      }

      .value-input {
        grid-area: value-input;
      }

      .key-error {
        grid-area: key-error;
        align-self: start;
      }

      .value-error {
        grid-area: value-error;
        align-self: start;
      }

      .remove {
        grid-area: remove;
        display: grid;
        place-items: center;
        width: 28px;
        height: 28px;
        padding: 0;
        border: 1px solid var(--ia-theme-field-border-color, #767676);
        border-radius: 3px;
        background: var(--ia-theme-secondary-background-color, #fff);
        color: var(--ia-theme-primary-text-color, #333);
        cursor: pointer;
      }

      .remove:hover:not(:disabled) {
        border-color: var(--ia-theme-color-danger-text, #c62828);
        color: var(--ia-theme-color-danger-text, #c62828);
      }

      .remove:disabled {
        cursor: default;
        opacity: 0.6;
      }

      .remove .icon {
        width: 12px;
        height: 12px;
      }

      @media (max-width: 767px) {
        .row {
          grid-template-columns: minmax(0, 1fr) 28px;
          grid-template-areas:
            'key-label .'
            'key-input remove'
            'key-error .'
            'value-label .'
            'value-input .'
            'value-error .';
        }
      }
    `]}};_([t({attribute:!1})],H.prototype,`rows`,void 0),_([t({attribute:!1})],H.prototype,`errors`,void 0),_([t({type:Boolean})],H.prototype,`disabled`,void 0),_([o()],H.prototype,`untouchedRowIds`,void 0),_([m(`#add`)],H.prototype,`addButton`,void 0),H=_([p(`upload-custom-metadata`)],H);var jr={p:`p`,br:`br`,strong:`strong`,b:`strong`,em:`em`,i:`em`,u:`u`,ul:`ul`,ol:`ol`,li:`li`,a:`a`},Mr=new Set([`script`,`style`,`template`,`iframe`,`object`,`embed`,`noscript`,`svg`,`math`,`textarea`,`select`,`title`]);function Nr(e){let t=new DOMParser().parseFromString(e,`text/html`),n=t.createElement(`div`);return Pr(t.body,n),ee(n.innerHTML.trim())}function Pr(e,t){let n=t.ownerDocument;for(let r of e.childNodes){if(r.nodeType===Node.TEXT_NODE){t.append(n.createTextNode(r.textContent??``));continue}if(!(r instanceof Element))continue;let e=r.localName;if(Mr.has(e))continue;let i=jr[e],a=r.getAttribute(`href`)??``,o=i===`a`&&!b(a);if(!i||o){Pr(r,t);continue}let s=n.createElement(i);i===`a`&&s.setAttribute(`href`,a.trim()),Pr(r,s),t.append(s)}}function Fr(e){let{selection:t,doc:n}=e,r=!1,i=!1;return n.nodesBetween(t.from,t.to,e=>{e.type.name===`listItem`&&(r=!0),e.isText&&e.marks.length>0&&(i=!0)}),r?!0:t.empty?(e.storedMarks??t.$from.marks()).length>0:i}function Ir(e){let t=e.schema.marks.link;if(!t)return!1;let{empty:n,from:r,to:i,$from:a}=e.selection;return n?t.isInSet(a.marks())!==void 0:e.doc.rangeHasMark(r,i,t)}function Lr(e,t,n){if(!(n<=0))switch(e){case`ArrowRight`:return(t+1)%n;case`ArrowLeft`:return(t-1+n)%n;case`Home`:return 0;case`End`:return n-1;default:return}}function Rr(e){let t=e.trim();if(!(t===``||/\s/.test(t)))return/^[a-z][a-z\d+.-]*:/i.test(t)?b(t)?t:void 0:/^[^@/]+@[^@/]+\.[^@/]+$/.test(t)?`mailto:${t}`:/^[^/?#]+\.[^/?#]+/.test(t)?`https://${t}`:void 0}var U=class extends s{constructor(...e){super(...e),this.href=``,this.canRemove=!1,this.invalid=!1}firstUpdated(){this.input?.focus(),this.input?.select()}render(){return i`
      <div
        class="popover"
        role="dialog"
        aria-label=${this.canRemove?a(`Edit link`):a(`Add link`)}
        @keydown=${this.handleKeydown}
      >
        <label class="small-label" for="url">${a(`Link address`)}</label>
        <input
          id="url"
          class=${l({control:!0,danger:this.invalid})}
          type="url"
          inputmode="url"
          spellcheck="false"
          autocapitalize="none"
          autocomplete="off"
          placeholder=${a(`https://`,{desc:`Placeholder of the link address box; usually kept as is.`})}
          .value=${this.href}
          aria-invalid=${f(this.invalid?`true`:void 0)}
          aria-describedby=${f(this.invalid?`url-error`:void 0)}
          @input=${this.handleInput}
        />
        ${this.invalid?i`<p class="error" id="url-error">
              ${R}${a(`Enter a web address starting with http:// or https://, or an email address.`)}
            </p>`:n}
        <div class="actions">
          <button
            type="button"
            class="button-primary apply"
            @click=${this.apply}
          >
            ${a(`Apply`)}
          </button>
          ${this.canRemove?i`<button
                type="button"
                class="button-secondary remove-link"
                @click=${this.removeLink}
              >
                ${a(`Remove`)}
              </button>`:n}
          <button
            type="button"
            class="button-secondary cancel"
            @click=${this.cancel}
          >
            ${a(`Cancel`)}
          </button>
        </div>
      </div>
    `}handleInput(){this.invalid=!1}handleKeydown(e){if(e.key===`Escape`){e.preventDefault(),e.stopPropagation(),this.cancel();return}e.key===`Enter`&&e.composedPath()[0]===this.input&&(e.preventDefault(),this.apply())}apply(){let e=Rr(this.input?.value??``);if(!e){this.invalid=!0,this.input?.focus();return}let t=new CustomEvent(`linkApplied`,{detail:{href:e}});this.dispatchEvent(t)}removeLink(){this.dispatchEvent(new CustomEvent(`linkRemoved`))}cancel(){this.dispatchEvent(new CustomEvent(`linkCancelled`))}static get styles(){return[L,e`
      :host {
        display: block;
      }

      .popover {
        display: grid;
        gap: 6px;
        padding: 10px 12px 12px;
        border-bottom: 1px solid var(--ia-theme-border-color, #ccc);
        background: var(--ia-theme-tertiary-background-color, #efefef);
      }

      .small-label {
        font-size: 13px;
        font-weight: 700;
      }

      .actions {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-top: 4px;
      }

      .button-primary {
        display: inline-flex;
        align-items: center;
        height: 34px;
        padding: 0 14px;
        border: 1px solid var(--upload-primary-button-border, #3b52e6);
        border-radius: 4px;
        background: var(--upload-primary-button-fill, #4b64ff);
        color: #fff;
        font: inherit;
        font-size: 14px;
        font-weight: 700;
        cursor: pointer;
      }

      .button-primary:hover {
        background: var(--upload-primary-button-hover-fill, #3b52e6);
      }
    `]}};_([t({type:String})],U.prototype,`href`,void 0),_([t({type:Boolean})],U.prototype,`canRemove`,void 0),_([o()],U.prototype,`invalid`,void 0),_([m(`#url`)],U.prototype,`input`,void 0),U=_([p(`upload-link-popover`)],U);var zr={bold:!1,italic:!1,underline:!1,orderedList:!1,bulletList:!1,link:!1},Br={removeLink:!1,clearFormatting:!1},W=class extends s{constructor(...e){super(...e),this.value=``,this.fieldState=`neutral`,this.disabled=!1,this.loaded=!1,this.loadFailed=!1,this.sourceMode=!1,this.sourceText=``,this.isEmpty=!0,this.active=zr,this.selectionTools=Br,this.linkBoxOpen=!1,this.linkHref=``,this.linkCanRemove=!1,this.toolbarIndex=0,this.ready=new Promise(e=>{this.resolveReady=e})}async focusControl(){if(await this.ready,this.sourceMode||this.loadFailed){this.sourceTextarea?.focus();return}this.editor?.view.focus()}connectedCallback(){super.connectedCallback(),this.hasUpdated&&!this.editor&&this.loadEditor()}disconnectedCallback(){super.disconnectedCallback(),this.editor?.destroy(),this.editor=void 0,this.loaded=!1}firstUpdated(){this.loadEditor()}willUpdate(e){let{editor:t}=this;t&&(e.has(`value`)&&!this.sourceMode&&this.value!==y(t)&&t.commands.setContent(this.value,{emitUpdate:!1}),e.has(`disabled`)&&t.setEditable(!this.disabled,!1),e.has(`fieldState`)&&t.setOptions({editorProps:{...t.options.editorProps,attributes:this.editorAttributes}}))}render(){let e=this.fieldState===`missing`,t=this.sourceMode||this.loadFailed,r=!this.loaded&&!this.loadFailed,o=this.loaded&&!this.sourceMode&&this.isEmpty;return i`
      <div class="field">
        <div class="label-row">
          <span class="label" id="label" @click=${this.focusControl}
            >${a(`Description`)}</span
          >
          ${B({id:`state`,state:this.fieldState})}
        </div>
        <div
          class=${l({box:!0,danger:e,locked:this.disabled})}
        >
          ${this.toolbarTemplate}
          ${this.linkBoxOpen?i`<upload-link-popover
                .href=${this.linkHref}
                ?canRemove=${this.linkCanRemove}
                @linkApplied=${this.handleLinkApplied}
                @linkRemoved=${this.handleLinkRemoved}
                @linkCancelled=${this.handleLinkCancelled}
              ></upload-link-popover>`:n}
          <div class="area">
            <div
              id="editor"
              class="editor-host"
              ?hidden=${!this.loaded||this.sourceMode}
            ></div>
            ${o?i`<div class="placeholder" aria-hidden="true">
                  ${this.placeholderText}
                </div>`:n}
            ${r?i`<div class="loading" aria-hidden="true">
                  ${this.placeholderText}
                </div>`:n}
            <textarea
              id="source"
              class="source"
              ?hidden=${!t}
              .value=${u(this.sourceText)}
              placeholder=${f(this.loadFailed?this.placeholderText:void 0)}
              spellcheck="false"
              aria-labelledby="label source-note"
              aria-describedby="state"
              aria-required="true"
              aria-invalid=${f(e?`true`:void 0)}
              ?readonly=${this.disabled}
              @input=${this.handleSourceInput}
            ></textarea>
            <span id="source-note" hidden>${a(`HTML source`)}</span>
          </div>
        </div>
      </div>
    `}get editorAttributes(){let e={role:`textbox`,"aria-multiline":`true`,"aria-labelledby":`label`,"aria-describedby":`state`,"aria-required":`true`,"aria-placeholder":this.placeholderText};return this.fieldState===`missing`&&(e[`aria-invalid`]=`true`),e}get placeholderText(){return a(`What is this item? Be as detailed as you can, as it makes searching for it easier!`)}get toolbarButtons(){let e=this.loaded&&!this.disabled,t=e&&!this.sourceMode,n=e=>t=>this.toggleFormat(e,t);return[{id:`bold-button`,label:a(`Bold`),content:i`<b aria-hidden="true">B</b>`,pressed:this.active.bold,available:t,startsGroup:!1,keepsEditorFocus:!0,activate:n(`bold`)},{id:`italic-button`,label:a(`Italic`),content:i`<i aria-hidden="true">I</i>`,pressed:this.active.italic,available:t,startsGroup:!1,keepsEditorFocus:!0,activate:n(`italic`)},{id:`underline-button`,label:a(`Underline`),content:i`<u aria-hidden="true">U</u>`,pressed:this.active.underline,available:t,startsGroup:!1,keepsEditorFocus:!0,activate:n(`underline`)},{id:`numbered-list-button`,label:a(`Numbered list`),content:or,pressed:this.active.orderedList,available:t,startsGroup:!0,keepsEditorFocus:!0,activate:n(`orderedList`)},{id:`bulleted-list-button`,label:a(`Bulleted list`),content:sr,pressed:this.active.bulletList,available:t,startsGroup:!1,keepsEditorFocus:!0,activate:n(`bulletList`)},{id:`link-button`,label:a(`Add link`),content:cr,expanded:this.linkBoxOpen,available:t,startsGroup:!0,keepsEditorFocus:!1,activate:()=>this.openLinkBox()},{id:`remove-link-button`,label:a(`Remove link`),content:lr,available:t&&this.selectionTools.removeLink,startsGroup:!1,keepsEditorFocus:!0,activate:e=>this.removeLink(e)},{id:`clear-formatting-button`,label:a(`Clear formatting`),content:ur,available:t&&this.selectionTools.clearFormatting,startsGroup:!0,keepsEditorFocus:!0,activate:e=>this.clearFormatting(e)},{id:`source-button`,label:a(`Edit HTML source`),content:i`<span class="source-label" aria-hidden="true"
          >${a(`HTML`)}</span
        >`,pressed:this.sourceMode,available:e,startsGroup:!0,keepsEditorFocus:!1,activate:()=>this.toggleSource()}]}get toolbarTemplate(){return i`
      <div
        class="toolbar"
        role="toolbar"
        aria-label=${a(`Description formatting`)}
        aria-controls="editor"
        @keydown=${this.handleToolbarKeydown}
      >
        ${this.toolbarButtons.map((e,t)=>i`
            ${e.startsGroup?i`<span class="separator" aria-hidden="true"></span>`:n}
            <button
              type="button"
              id=${e.id}
              class="tool"
              title=${e.label}
              aria-pressed=${f(e.pressed===void 0?void 0:String(e.pressed))}
              aria-expanded=${f(e.expanded===void 0?void 0:String(e.expanded))}
              aria-disabled=${f(e.available?void 0:`true`)}
              tabindex=${t===this.toolbarIndex?`0`:`-1`}
              @focus=${()=>this.toolbarIndex=t}
              @mousedown=${t=>{e.keepsEditorFocus&&t.preventDefault()}}
              @click=${t=>{e.available&&e.activate(t)}}
            >
              ${e.content}
            </button>
          `)}
      </div>
    `}handleToolbarKeydown(e){let t=[...this.shadowRoot?.querySelectorAll(`.tool`)??[]],n=Lr(e.key,this.toolbarIndex,t.length);n!==void 0&&(e.preventDefault(),this.toolbarIndex=n,t[n]?.focus())}async loadEditor(){let e=await h(()=>g(()=>import(`./create-description-editor-CkLBnKZw.js`),__vite__mapDeps([0,1]))).catch(()=>void 0);if(!e){this.sourceText=this.value,this.loadFailed=!0,this.emitIfChanged(Nr(this.value)),this.resolveReady?.();return}let t=this.editorHost;!this.isConnected||this.editor||!t||(t.replaceChildren(),this.editor=e.createDescriptionEditor({element:t,content:this.value,editable:!this.disabled,attributes:this.editorAttributes,onChange:e=>this.emitDescription(e),onTransaction:()=>this.syncFromEditor()}),this.emitIfChanged(Nr(this.value)),this.syncFromEditor(),this.loaded=!0,this.resolveReady?.())}syncFromEditor(){let{editor:e}=this;e&&(this.isEmpty=e.isEmpty,this.active={bold:e.isActive(`bold`),italic:e.isActive(`italic`),underline:e.isActive(`underline`),orderedList:e.isActive(`orderedList`),bulletList:e.isActive(`bulletList`),link:e.isActive(`link`)},this.selectionTools={removeLink:Ir(e.state),clearFormatting:Fr(e.state)})}toolChain(e,t){return t.detail>0?e.chain().focus():e.chain()}toggleFormat(e,t){let{editor:n}=this;if(!n)return;let r=this.toolChain(n,t);({bold:e=>e.toggleBold(),italic:e=>e.toggleItalic(),underline:e=>e.toggleUnderline(),orderedList:e=>e.toggleOrderedList(),bulletList:e=>e.toggleBulletList()})[e](r).run()}removeLink(e){let{editor:t}=this;t&&(this.linkBoxOpen=!1,this.toolChain(t,e).extendMarkRange(`link`).unsetLink().run())}clearFormatting(e){let{editor:t}=this;t&&(this.linkBoxOpen=!1,this.toolChain(t,e).clearNodes().unsetAllMarks().command(({tr:e})=>(e.setStoredMarks([]),!0)).run())}openLinkBox(){let{editor:e}=this;if(!e)return;if(this.linkBoxOpen){this.linkBoxOpen=!1;return}let t=e.getAttributes(`link`).href;this.linkHref=typeof t==`string`?t:``,this.linkCanRemove=e.isActive(`link`),this.linkBoxOpen=!0}handleLinkApplied(e){let{editor:t}=this;if(this.linkBoxOpen=!1,!t)return;let{href:n}=e.detail;if(t.state.selection.empty&&!t.isActive(`link`)){t.chain().focus().insertContent({type:`text`,text:n,marks:[{type:`link`,attrs:{href:n}}]}).run();return}t.chain().focus().extendMarkRange(`link`).setLink({href:n}).run()}handleLinkRemoved(){this.linkBoxOpen=!1,this.editor?.chain().focus().extendMarkRange(`link`).unsetLink().run()}async handleLinkCancelled(){this.linkBoxOpen=!1,await this.updateComplete,this.linkButton?.focus()}toggleSource(){let{editor:e}=this;if(!e)return;if(!this.sourceMode){this.sourceText=y(e),this.linkBoxOpen=!1,this.sourceMode=!0;return}e.commands.setContent(this.sourceText,{emitUpdate:!1}),this.sourceMode=!1;let t=y(e);t!==this.value&&this.emitDescription(t)}handleSourceInput(e){if(!(e.target instanceof HTMLTextAreaElement))return;this.sourceText=e.target.value;let{editor:t}=this;if(!t){this.emitDescription(Nr(this.sourceText));return}t.commands.setContent(this.sourceText,{emitUpdate:!1}),this.emitDescription(y(t))}emitIfChanged(e){e!==this.value&&this.emitDescription(e)}emitDescription(e){this.dispatchEvent(D(`metadataChanged`,{changes:{description:e}}))}static get styles(){return[L,e`
      .ProseMirror {
        position: relative;
        word-wrap: break-word;
        white-space: pre-wrap;
        white-space: break-spaces;
        -webkit-font-variant-ligatures: none;
        font-variant-ligatures: none;
        font-feature-settings: 'liga' 0;
      }

      .ProseMirror [contenteditable='false'] {
        white-space: normal;
      }

      .ProseMirror [contenteditable='false'] [contenteditable='true'] {
        white-space: pre-wrap;
      }

      .ProseMirror pre {
        white-space: pre-wrap;
      }

      img.ProseMirror-separator {
        display: inline !important;
        border: none !important;
        margin: 0 !important;
        width: 0 !important;
        height: 0 !important;
      }

      .ProseMirror-hideselection *::selection {
        background: transparent;
      }

      .ProseMirror-hideselection * {
        caret-color: transparent;
      }
    `,e`
      :host {
        --upload-editor-min-height: calc(4 * 1.4em + 16px);
      }

      .label {
        cursor: default;
      }

      .box {
        overflow: hidden;
        border: 1px solid var(--ia-theme-field-border-color, #767676);
        border-radius: 3px;
        background: var(--ia-theme-secondary-background-color, #fff);
      }

      .box.danger {
        border: 2px solid var(--ia-theme-color-danger-text, #c62828);
      }

      /* Locked while uploading, as the other fields: dashed and muted. */
      .box.locked {
        border-style: dashed;
      }

      .box.locked .ProseMirror,
      .box.locked .source {
        color: var(--ia-theme-secondary-text-color, #595959);
        cursor: default;
      }

      /* One ring around toolbar and text while the text has focus. */
      .box:has(.ProseMirror:focus, .source:focus) {
        outline: 2px solid var(--ia-theme-link-color, #4b64ff);
        outline-offset: 2px;
      }

      .toolbar {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 2px;
        padding: 4px;
        border-bottom: 1px solid var(--ia-theme-border-color, #ccc);
        border-radius: 2px 2px 0 0;
        background: color-mix(
          in srgb,
          var(--ia-theme-primary-text-color, #333) 4%,
          var(--ia-theme-secondary-background-color, #fff)
        );
      }

      .tool {
        display: grid;
        place-items: center;
        min-width: 30px;
        height: 30px;
        padding: 0 6px;
        border: 0;
        border-radius: 3px;
        background: none;
        color: var(--ia-theme-primary-text-color, #333);
        font: inherit;
        font-size: 14px;
        line-height: 1;
        cursor: pointer;
      }

      .tool:hover:not([aria-disabled='true']) {
        background: var(--ia-theme-tertiary-background-color, #efefef);
      }

      /* Pressed shows as a tinted, outlined button, not colour alone. */
      .tool[aria-pressed='true'],
      .tool[aria-expanded='true'] {
        background: color-mix(
          in srgb,
          var(--ia-theme-link-color, #4b64ff) 16%,
          var(--ia-theme-secondary-background-color, #fff)
        );
        box-shadow: inset 0 0 0 1px var(--ia-theme-link-color, #4b64ff);
      }

      .tool[aria-disabled='true'] {
        opacity: 0.45;
        cursor: default;
      }

      .tool .icon {
        width: 16px;
        height: 16px;
      }

      .source-label {
        font-size: 12px;
        font-weight: 700;
        letter-spacing: 0.02em;
      }

      .separator {
        align-self: stretch;
        width: 1px;
        margin: 4px 3px;
        background: var(--ia-theme-border-color, #ccc);
      }

      .area {
        position: relative;
      }

      .ProseMirror,
      .loading,
      .placeholder,
      .source {
        min-height: var(--upload-editor-min-height);
        padding: 8px 10px;
        font-size: 15px;
        line-height: 1.4;
      }

      .ProseMirror {
        max-height: 360px;
        overflow-y: auto;
        color: var(--ia-theme-primary-text-color, #333);
        outline: none;
      }

      .ProseMirror p {
        margin: 0 0 0.5em;
      }

      .ProseMirror ul,
      .ProseMirror ol {
        margin: 0 0 0.5em;
        padding-left: 1.6em;
      }

      .ProseMirror li p {
        margin: 0;
      }

      .ProseMirror > :last-child {
        margin-bottom: 0;
      }

      .ProseMirror a {
        color: var(--ia-theme-link-color, #4b64ff);
        text-decoration: underline;
      }

      .loading,
      .placeholder {
        color: var(--ia-theme-secondary-text-color, #6b6b6b);
      }

      .placeholder {
        position: absolute;
        inset: 0 0 auto;
        pointer-events: none;
      }

      .source {
        display: block;
        width: 100%;
        margin: 0;
        border: 0;
        background: transparent;
        color: var(--ia-theme-primary-text-color, #333);
        font-family: var(
          --ia-theme-monospace-font-family,
          ui-monospace,
          Menlo,
          monospace
        );
        font-size: 13px;
        resize: vertical;
        outline: none;
      }

      @media (max-width: 767px) {
        .ProseMirror,
        .loading,
        .placeholder,
        .source {
          font-size: 16px;
        }
      }
    `]}};_([t({type:String})],W.prototype,`value`,void 0),_([t({type:String})],W.prototype,`fieldState`,void 0),_([t({type:Boolean})],W.prototype,`disabled`,void 0),_([o()],W.prototype,`loaded`,void 0),_([o()],W.prototype,`loadFailed`,void 0),_([o()],W.prototype,`sourceMode`,void 0),_([o()],W.prototype,`sourceText`,void 0),_([o()],W.prototype,`isEmpty`,void 0),_([o()],W.prototype,`active`,void 0),_([o()],W.prototype,`selectionTools`,void 0),_([o()],W.prototype,`linkBoxOpen`,void 0),_([o()],W.prototype,`linkHref`,void 0),_([o()],W.prototype,`linkCanRemove`,void 0),_([o()],W.prototype,`toolbarIndex`,void 0),_([m(`#editor`)],W.prototype,`editorHost`,void 0),_([m(`#source`)],W.prototype,`sourceTextarea`,void 0),_([m(`#link-button`)],W.prototype,`linkButton`,void 0),W=_([p(`upload-description-editor`)],W);var G=class extends s{constructor(...e){super(...e),this.heading=``,this.summary=``,this.open=!1}render(){return i`
      <h2 class="heading">
        <button
          type="button"
          class="bar"
          aria-expanded=${this.open?`true`:`false`}
          aria-controls="body"
          @click=${this.handleBarClick}
        >
          ${rr}
          <span class="title">${this.heading}</span>
          <span class="summary">${this.summary}</span>
        </button>
      </h2>
      <div class="body" id="body" ?hidden=${!this.open}>
        <slot></slot>
      </div>
    `}handleBarClick(){let e=new CustomEvent(`sectionToggled`,{detail:{open:!this.open}});this.dispatchEvent(e)}static get styles(){return e`
      :host {
        --upload-section-bar-fill--: var(
          --upload-section-bar-fill,
          var(--ia-theme-primary-text-color, #333)
        );
        --upload-section-bar-text--: var(
          --upload-section-bar-text,
          var(--ia-theme-secondary-background-color, #fff)
        );
        display: block;
      }

      [hidden] {
        display: none !important;
      }

      .heading {
        margin: 0;
        font: inherit;
      }

      .bar {
        display: flex;
        align-items: center;
        gap: 10px;
        width: 100%;
        min-height: 32px;
        padding: 6px 10px;
        border: 0;
        border-radius: 3px;
        background: var(--upload-section-bar-fill--);
        color: var(--upload-section-bar-text--);
        font: inherit;
        font-size: 14px;
        font-weight: 700;
        line-height: 20px;
        text-align: left;
        cursor: pointer;
      }

      .bar:hover {
        background: color-mix(
          in srgb,
          var(--upload-section-bar-fill--) 86%,
          var(--upload-section-bar-text--)
        );
      }

      .bar:focus-visible {
        outline: 2px solid var(--ia-theme-link-color, #4b64ff);
        outline-offset: 2px;
      }

      :host([open]) .bar {
        border-radius: 3px 3px 0 0;
      }

      .disclosure {
        width: 10px;
        height: 10px;
        flex: none;
        transition: transform 0.15s ease;
      }

      .bar[aria-expanded='true'] .disclosure {
        transform: rotate(90deg);
      }

      /* About the design's #ccc on #333, from the bar's own colours. */
      .summary {
        margin-left: auto;
        color: color-mix(
          in srgb,
          var(--upload-section-bar-text--) 78%,
          var(--upload-section-bar-fill--)
        );
        font-size: 13px;
        font-weight: 400;
      }

      .body {
        padding: 20px 22px 22px;
        border: 1px solid var(--ia-theme-border-color, #ccc);
        border-top: 0;
        border-radius: 0 0 3px 3px;
        background: var(--ia-theme-tertiary-background-color, #efefef);
      }

      @media (max-width: 767px) {
        .bar {
          flex-wrap: wrap;
          row-gap: 0;
        }

        /* Lined up with the title text, past the triangle and gap. */
        .summary {
          flex-basis: 100%;
          margin-left: 20px;
        }

        .body {
          padding: 16px 14px 18px;
        }
      }

      @media (prefers-reduced-motion: reduce) {
        .disclosure {
          transition: none;
        }
      }
    `}};_([t({type:String})],G.prototype,`heading`,void 0),_([t({type:String})],G.prototype,`summary`,void 0),_([t({type:Boolean,reflect:!0})],G.prototype,`open`,void 0),G=_([p(`upload-form-section`)],G);var Vr=3,K=class extends s{constructor(...e){super(...e),this.value=``,this.status={state:`idle`},this.fieldState=`neutral`,this.maxLength=80,this.disabled=!1,this.refocusInput=!1}focusControl(){this.input?.focus()}willUpdate(e){e.has(`value`)&&this.value!==this.userCheckedValue&&(this.userCheckedValue=void 0)}updated(e){e.has(`value`)&&this.refocusInput&&(this.refocusInput=!1,this.input?.focus())}render(){let{status:e}=this,t=e.state===`taken`||e.state===`illegal`||e.state===`error`,o=e.state===`idle`&&this.fieldState===`missing`,s=t||o,c=e.state===`taken`||e.state===`illegal`||o,d=e.state===`illegal`,p=[`prefix-note`,`state`,d?`rule`:``,`help`].filter(e=>e!==``).join(` `);return i`
      <div class="field">
        <div class="label-row">
          <label class="label" for="control">${a(`Page URL`)}</label>
          ${this.stateTemplate}
        </div>
        <div
          class=${l({prefix:!0,danger:s,locked:this.disabled})}
        >
          <span class="prefix-text" aria-hidden="true"
            >archive.org/details/</span
          >
          <input
            id="control"
            class="prefix-input"
            type="text"
            spellcheck="false"
            autocapitalize="none"
            autocomplete="off"
            .value=${u(this.value)}
            maxlength=${this.maxLength}
            aria-required="true"
            aria-invalid=${f(c?`true`:void 0)}
            aria-describedby=${p}
            ?readonly=${this.disabled}
            @input=${this.handleInput}
          />
        </div>
        <span id="prefix-note" hidden
          >${a(`Address starts with archive.org/details/`)}</span
        >
        ${d?i`<p class="error" id="rule">
              ${R}${a(r`Use 5 to ${this.maxLength} characters: letters, numbers, periods, hyphens and underscores, starting with a letter or number.`)}
            </p>`:n}
        ${this.suggestionsTemplate}
        <p class="help" id="help">
          ${a(`This identifier cannot be changed after the item is created.`)}
        </p>
        <span class="sr-only" role="status">${this.announcement}</span>
      </div>
    `}get stateTemplate(){let{status:e}=this;switch(e.state){case`idle`:return B({id:`state`,state:this.fieldState,satisfiedText:a(`available`)});case`checking`:return i`<span class="state neutral" id="state"
          ><span class="spinner" aria-hidden="true"></span>${a(`Checking…`)}</span
        >`;case`available`:return i`<span class="state satisfied" id="state"
          >${er}${a(`available`)}</span
        >`;case`taken`:return i`<span class="state missing" id="state"
          >${R}${a(`Taken`)}</span
        >`;case`illegal`:return i`<span class="state missing" id="state"
          >${R}${a(`Not allowed`)}</span
        >`;case`error`:return i`<span class="state missing" id="state"
          >${R}<button
            type="button"
            class="link-button retry"
            ?disabled=${this.disabled}
            @click=${this.handleRetry}
          >
            ${a(`Couldn't check. Try again`)}
          </button></span
        >`}}get suggestionsTemplate(){let{status:e}=this;if(!((e.state===`taken`||e.state===`illegal`)&&e.suggestions.length>0)||this.disabled)return n;let t=e.suggestions.slice(0,Vr);return i`
      <div
        class="suggestions"
        role="group"
        aria-label=${a(`Available Page URLs`)}
      >
        ${t.map(e=>i`
            <button
              type="button"
              class="button-secondary suggestion"
              @click=${()=>this.chooseSuggestion(e)}
            >
              ${a(r`Use ${e}`)}
            </button>
          `)}
      </div>
    `}get announcement(){let{status:e}=this;if(e.state===`idle`||e.state===`checking`)return``;let t=e.identifier===this.userCheckedValue,n=e.state===`taken`&&e.identifier===this.value;if(!t&&!n)return``;switch(e.state){case`available`:return a(r`${e.identifier} is available.`);case`taken`:return a(r`${e.identifier} is taken.`);case`illegal`:return a(r`${e.identifier} is not allowed.`);case`error`:return a(`Couldn't check the Page URL.`)}}handleInput(e){if(!(e.target instanceof HTMLInputElement))return;let t=e.target.value;this.userCheckedValue=t,this.dispatchEvent(D(`metadataChanged`,{changes:{identifier:t}}))}handleRetry(){this.userCheckedValue=this.value,this.dispatchEvent(D(`metadataChanged`,{changes:{identifier:this.value}}))}chooseSuggestion(e){this.refocusInput=!0,this.userCheckedValue=e,this.dispatchEvent(D(`identifierSuggestionChosen`,{identifier:e}))}static get styles(){return[L,e`
      .prefix {
        display: flex;
        overflow: hidden;
        border: 1px solid var(--ia-theme-field-border-color, #767676);
        border-radius: 3px;
        background: var(--ia-theme-secondary-background-color, #fff);
      }

      .prefix.danger {
        border: 2px solid var(--ia-theme-color-danger-text, #c62828);
      }

      .prefix:focus-within {
        outline: 2px solid var(--ia-theme-link-color, #4b64ff);
        outline-offset: 2px;
      }

      /* Locked while uploading, as the other fields: dashed and muted. */
      .prefix.locked {
        border-style: dashed;
      }

      .prefix.locked .prefix-input {
        color: var(--ia-theme-secondary-text-color, #595959);
        cursor: default;
      }

      .prefix-text {
        display: flex;
        align-items: center;
        padding: 0 10px;
        border-right: 1px solid var(--ia-theme-field-border-color, #767676);
        background: color-mix(
          in srgb,
          var(--ia-theme-primary-text-color, #333) 7%,
          var(--ia-theme-secondary-background-color, #fff)
        );
        color: var(--ia-theme-secondary-text-color, #555);
        font-size: 15px;
        white-space: nowrap;
      }

      .prefix-input {
        flex: 1;
        min-width: 0;
        height: 36px;
        margin: 0;
        padding: 0 10px;
        border: 0;
        border-radius: 0;
        background: transparent;
        color: var(--ia-theme-primary-text-color, #333);
        font: inherit;
        font-size: 15px;
      }

      .prefix.danger .prefix-input {
        height: 34px;
      }

      .prefix-input:focus-visible {
        outline: none;
      }

      .spinner {
        width: 13px;
        height: 13px;
        flex: none;
        border: 2px solid currentColor;
        border-right-color: transparent;
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
      }

      @keyframes spin {
        to {
          transform: rotate(360deg);
        }
      }

      .retry {
        font-size: 13px;
        color: var(--ia-theme-color-danger-text, #c62828);
      }

      .suggestions {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }

      .suggestion {
        max-width: 100%;
        overflow: hidden;
        text-overflow: ellipsis;
      }

      @media (max-width: 767px) {
        .prefix-text,
        .prefix-input {
          font-size: 16px;
        }
      }

      @media (prefers-reduced-motion: reduce) {
        .spinner {
          animation: none;
        }
      }
    `]}};_([t({type:String})],K.prototype,`value`,void 0),_([t({attribute:!1})],K.prototype,`status`,void 0),_([t({type:String})],K.prototype,`fieldState`,void 0),_([t({type:Number})],K.prototype,`maxLength`,void 0),_([t({type:Boolean})],K.prototype,`disabled`,void 0),_([m(`#control`)],K.prototype,`input`,void 0),K=_([p(`upload-page-url-field`)],K);var q=class extends s{constructor(...e){super(...e),this.name=`language`,this.label=``,this.value=``,this.options=[],this.state=`optional`,this.help=``,this.required=!1,this.disabled=!1}focusControl(){this.select?.focus()}render(){let e=this.help!==``,t=e?`state help`:`state`,r=this.state===`missing`,a=this.flatOptions.some(e=>!e.disabled&&e.value===this.value)?this.value:``;return i`
      <div class="field">
        <div class="label-row">
          <label class="label" for="control">${this.label}</label>
          ${B({id:`state`,state:this.state})}
        </div>
        <div class="select-wrap">
          <select
            id="control"
            class=${l({control:!0,danger:r})}
            .value=${u(a)}
            ?required=${this.required}
            aria-required=${f(this.required?`true`:void 0)}
            aria-invalid=${r?`true`:`false`}
            aria-describedby=${t}
            ?disabled=${this.disabled}
            @change=${this.handleChange}
          >
            ${d(this.options,(e,t)=>`groupLabel`in e?`group:${e.groupLabel}`:`${t}:${e.value}`,e=>`groupLabel`in e?i`<optgroup label=${e.groupLabel}>
                      ${d(e.options,(e,t)=>`${t}:${e.value}`,e=>this.optionTemplate(e,a))}
                    </optgroup>`:this.optionTemplate(e,a))}
          </select>
        </div>
        ${e?i`<p class="help" id="help">${this.help}</p>`:n}
      </div>
    `}get flatOptions(){return this.options.flatMap(e=>`groupLabel`in e?e.options:[e])}optionTemplate(e,t){let n=!e.disabled&&e.value===t;return i`
      <option
        .value=${e.value}
        ?disabled=${e.disabled??!1}
        ?selected=${n}
      >
        ${e.label}
      </option>
    `}handleChange(e){if(!(e.target instanceof HTMLSelectElement))return;let t={[this.name]:e.target.value};this.dispatchEvent(D(`metadataChanged`,{changes:t}))}static get styles(){return L}};_([t({type:String,reflect:!0})],q.prototype,`name`,void 0),_([t({type:String})],q.prototype,`label`,void 0),_([t({type:String})],q.prototype,`value`,void 0),_([t({attribute:!1})],q.prototype,`options`,void 0),_([t({type:String})],q.prototype,`state`,void 0),_([t({type:String})],q.prototype,`help`,void 0),_([t({type:Boolean})],q.prototype,`required`,void 0),_([t({type:Boolean})],q.prototype,`disabled`,void 0),_([m(`#control`)],q.prototype,`select`,void 0),q=_([p(`upload-select-field`)],q);var J=class extends s{constructor(...e){super(...e),this.name=`title`,this.label=``,this.value=``,this.state=`optional`,this.placeholder=``,this.help=``,this.error=``,this.required=!1,this.disabled=!1}focusControl(){this.input?.focus()}render(){let e=this.help!==``,t=this.error!==``,r=[`state`,e?`help`:``,t?`error`:``].filter(e=>e!==``).join(` `),a=this.state===`missing`||t;return i`
      <div class="field">
        <div class="label-row">
          <label class="label" for="control">${this.label}</label>
          ${B({id:`state`,state:this.state})}
        </div>
        <input
          id="control"
          class=${l({control:!0,danger:a})}
          type="text"
          .value=${u(this.value)}
          placeholder=${f(this.placeholder||void 0)}
          maxlength=${f(this.maxlength)}
          aria-required=${f(this.required?`true`:void 0)}
          aria-invalid=${f(a?`true`:void 0)}
          aria-describedby=${r}
          ?readonly=${this.disabled}
          @input=${this.handleInput}
        />
        ${e?i`<p class="help" id="help">${this.help}</p>`:n}
        ${t?i`<p class="error" id="error">${R}${this.error}</p>`:n}
      </div>
    `}handleInput(e){if(!(e.target instanceof HTMLInputElement))return;let t={[this.name]:e.target.value};this.dispatchEvent(D(`metadataChanged`,{changes:t}))}static get styles(){return L}};_([t({type:String,reflect:!0})],J.prototype,`name`,void 0),_([t({type:String})],J.prototype,`label`,void 0),_([t({type:String})],J.prototype,`value`,void 0),_([t({type:String})],J.prototype,`state`,void 0),_([t({type:String})],J.prototype,`placeholder`,void 0),_([t({type:String})],J.prototype,`help`,void 0),_([t({type:String})],J.prototype,`error`,void 0),_([t({type:Number})],J.prototype,`maxlength`,void 0),_([t({type:Boolean})],J.prototype,`required`,void 0),_([t({type:Boolean})],J.prototype,`disabled`,void 0),_([m(`#control`)],J.prototype,`input`,void 0),J=_([p(`upload-text-field`)],J);var Hr={title:``,identifier:``,description:``,subjects:``,collection:``,creator:``,date:``,language:``,licenseUrl:``,testItem:!1,custom:[]},Ur={title:`neutral`,identifier:`neutral`,description:`neutral`,subjects:`neutral`,collection:`neutral`},Y=class extends s{constructor(...e){super(...e),this.metadata=Hr,this.hasFiles=!1,this.fieldStates=Ur,this.identifierStatus={state:`idle`},this.identifierMaxLength=80,this.customErrors={},this.suggestedCollection=``,this.disabled=!1,this.lockedMessage=a(`Locked while uploading`),this.openSections={required:!0,recommended:!0,additional:!1},this.dateTouched=!1}openSection(e){this.openSections[e]||(this.openSections={...this.openSections,[e]:!0})}async focusField(e){e===`date`&&(this.dateTouched=!0);let t=hr(e);this.openSection(t),await this.updateComplete,await this.shadowRoot?.querySelector(`upload-form-section[data-section="${t}"]`)?.updateComplete,await(this.shadowRoot?.querySelector(`[data-field="${e}"]`))?.focusControl()}willUpdate(){this.dateError!==void 0&&$n(this.metadata.date)&&(this.dateTouched=!0)}render(){return i`
      <div class="sections">
        ${this.disabled?i`<p class="locked-note">${nr}${this.lockedMessage}</p>`:n}
        <upload-form-section
          data-section="required"
          heading=${a(`Required Information`)}
          summary=${mr(this.hasFiles,this.fieldStates)}
          ?open=${this.openSections.required}
          @sectionToggled=${e=>this.setSectionOpen(`required`,e)}
        >
          <div class="fields">${this.requiredFieldsTemplate}</div>
        </upload-form-section>

        <upload-form-section
          data-section="recommended"
          heading=${a(`Recommended Information`)}
          summary=${a(`Creator, Date, Language, License`)}
          ?open=${this.openSections.recommended}
          @sectionToggled=${e=>this.setSectionOpen(`recommended`,e)}
        >
          <div class="fields">${this.recommendedFieldsTemplate}</div>
        </upload-form-section>

        <upload-form-section
          data-section="additional"
          heading=${a(`Additional Information`)}
          summary=${this.context?.is_archive_user?a(`Test item, custom metadata`):a(`Custom metadata`)}
          ?open=${this.openSections.additional}
          @sectionToggled=${e=>this.setSectionOpen(`additional`,e)}
        >
          <div class="fields">${this.additionalFieldsTemplate}</div>
        </upload-form-section>
      </div>
    `}get requiredFieldsTemplate(){let{metadata:e,fieldStates:t,hasFiles:n}=this;return i`
      <upload-text-field
        data-field="title"
        name="title"
        label=${z(`title`)}
        .value=${e.title}
        state=${t.title}
        help=${a(n?`Prefilled from your largest file. Edit as needed.`:`Suggested from your largest file once you add files.`)}
        .maxlength=${255}
        required
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
      ></upload-text-field>

      <upload-page-url-field
        data-field="identifier"
        .value=${e.identifier}
        .status=${this.identifierStatus}
        fieldState=${t.identifier}
        .maxLength=${this.identifierMaxLength}
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
        @identifierSuggestionChosen=${this.relaySuggestionChosen}
      ></upload-page-url-field>

      <upload-description-editor
        data-field="description"
        .value=${e.description}
        fieldState=${t.description}
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
      ></upload-description-editor>

      <upload-text-field
        data-field="subjects"
        name="subjects"
        label=${z(`subjects`)}
        .value=${e.subjects}
        state=${t.subjects}
        placeholder=${a(`Keywords, separated by commas`)}
        .maxlength=${255}
        required
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
      ></upload-text-field>

      ${this.collectionFieldTemplate}
    `}get collectionFieldTemplate(){let{metadata:e,fieldStates:t}=this,n=this.context?.open_collections??[];if(this.context?.can_pick_any_collection){let r=this.hasFiles?a(`Chosen automatically from your file types. Type to search for another collection.`):a(`Chosen automatically once you add files. Type to search for another collection.`);return i`
        <upload-collection-combobox
          data-field="collection"
          .value=${e.collection}
          fieldState=${t.collection}
          help=${r}
          .openCollections=${n}
          suggestedCollection=${this.suggestedCollection}
          .searchCollections=${this.searchCollections}
          ?disabled=${this.disabled}
          @metadataChanged=${this.relayMetadataChanged}
        ></upload-collection-combobox>
      `}let r=this.hasFiles?a(`Chosen automatically from your file types.`):a(`Chosen automatically once you add files.`);return i`
      <upload-select-field
        data-field="collection"
        name="collection"
        label=${z(`collection`)}
        .value=${e.collection}
        .options=${Zn(n)}
        state=${t.collection}
        help=${r}
        required
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
      ></upload-select-field>
    `}get recommendedFieldsTemplate(){let{metadata:e,dateError:t}=this,n=this.dateTouched&&t?lt(t):``;return i`
      <upload-text-field
        data-field="creator"
        name="creator"
        label=${a(`Creator`)}
        .value=${e.creator}
        state="optional"
        placeholder=${a(`Person or organization who made this`)}
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
      ></upload-text-field>

      <upload-text-field
        data-field="date"
        name="date"
        label=${a(`Date`)}
        .value=${e.date}
        state="optional"
        placeholder=${a(`YYYY-MM-DD`)}
        help=${a(`When the item was created or published. A year on its own is fine.`)}
        error=${n}
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
        @focusout=${this.handleDateFocusOut}
      ></upload-text-field>

      <upload-select-field
        data-field="language"
        name="language"
        label=${a(`Language`)}
        .value=${e.language}
        .options=${pr(this.context?.languages??[])}
        state="optional"
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
      ></upload-select-field>

      <upload-select-field
        data-field="licenseUrl"
        name="licenseUrl"
        label=${a(`License`)}
        .value=${e.licenseUrl}
        .options=${Be()}
        state="optional"
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
      ></upload-select-field>
    `}get additionalFieldsTemplate(){return i`
      ${this.context?.is_archive_user?i`
            <div class="field">
              <label class="check">
                <input
                  type="checkbox"
                  id="test-item"
                  .checked=${u(this.metadata.testItem)}
                  aria-describedby="test-item-help"
                  ?disabled=${this.disabled}
                  @change=${this.handleTestItemChange}
                />
                ${a(`Test item`)}
              </label>
              <p class="help" id="test-item-help">
                ${a(`Removed automatically after 30 days.`)}
              </p>
            </div>
          `:n}
      <upload-custom-metadata
        data-field="custom"
        .rows=${this.metadata.custom}
        .errors=${this.customErrors}
        ?disabled=${this.disabled}
        @metadataChanged=${this.relayMetadataChanged}
      ></upload-custom-metadata>
    `}setSectionOpen(e,t){this.openSections={...this.openSections,[e]:t.detail.open}}handleDateFocusOut(){this.metadata.date.trim()!==``&&(this.dateTouched=!0)}handleTestItemChange(e){e.target instanceof HTMLInputElement&&this.dispatchEvent(D(`metadataChanged`,{changes:{testItem:e.target.checked}}))}relayMetadataChanged(e){this.dispatchEvent(D(`metadataChanged`,e.detail))}relaySuggestionChosen(e){this.dispatchEvent(D(`identifierSuggestionChosen`,e.detail))}static get styles(){return[L,e`
      :host {
        display: block;
        color: var(--ia-theme-primary-text-color, #333);
        font-family: var(
          --ia-theme-base-font-family,
          'Helvetica Neue',
          Helvetica,
          Arial,
          sans-serif
        );
        font-size: 14px;
        line-height: 1.42857;
      }

      .sections {
        display: flex;
        flex-direction: column;
        gap: 8px;
      }

      .fields {
        display: flex;
        flex-direction: column;
        gap: 20px;
      }

      .check {
        display: flex;
        align-items: center;
        gap: 10px;
        font-weight: 700;
      }

      .check input {
        width: 18px;
        height: 18px;
        margin: 0;
        accent-color: var(--upload-primary-button-fill, #4b64ff);
      }

      .check:has(input:disabled) {
        color: var(--ia-theme-secondary-text-color, #595959);
        cursor: default;
      }

      /* Said once, in words, for the whole form; each control also looks
         locked (see fieldStyles). */
      .locked-note {
        display: flex;
        align-items: center;
        gap: 6px;
        margin: 0;
        padding: 8px 12px;
        border: 1px dashed var(--ia-theme-field-border-color, #767676);
        border-radius: 3px;
        color: var(--ia-theme-secondary-text-color, #595959);
        font-size: 13px;
        font-weight: 700;
      }
    `]}};_([t({type:Object})],Y.prototype,`metadata`,void 0),_([t({type:Object})],Y.prototype,`context`,void 0),_([t({type:Boolean})],Y.prototype,`hasFiles`,void 0),_([t({type:Object})],Y.prototype,`fieldStates`,void 0),_([t({type:Object})],Y.prototype,`identifierStatus`,void 0),_([t({type:Number})],Y.prototype,`identifierMaxLength`,void 0),_([t({attribute:!1})],Y.prototype,`dateError`,void 0),_([t({type:Object})],Y.prototype,`customErrors`,void 0),_([t({attribute:!1})],Y.prototype,`searchCollections`,void 0),_([t({type:String})],Y.prototype,`suggestedCollection`,void 0),_([t({type:Boolean,reflect:!0})],Y.prototype,`disabled`,void 0),_([t({type:String})],Y.prototype,`lockedMessage`,void 0),_([o()],Y.prototype,`openSections`,void 0),_([o()],Y.prototype,`dateTouched`,void 0),Y=_([p(`upload-metadata-form`)],Y);function Wr(e){switch(e.stage){case`transferFailed`:return e.nothingUploaded?`backToForm`:void 0;case`taskFailed`:return`close`;default:return}}function Gr(e){let{retryable:t,nothingUploaded:n}=e,r=e.failedDuring===`identifierCheck`?`tryAgain`:`resume`,i=e.failedDuring===`signing`&&e.httpStatus===403;return{retry:t?r:void 0,backToForm:n,classicUploader:i,itemLink:!t&&!n}}var X=class extends s{constructor(...e){super(...e),this.runState={stage:`idle`},this.files=[],this.classicUploaderUrl=Xe,this.redirectCancelled=!1,this.reducedMotion=!1,this.announcement=``,this.openingAnnouncement=``,this.announcing=!1}focusHeading(){this.stageHeading?.focus()}willUpdate(){this.hasRun&&!this.dialog?.open?(this.openingAnnouncement=this.announcement,this.announcing=!1):this.announcement!==this.openingAnnouncement&&(this.announcing=!0)}render(){if(!this.hasRun)return n;let e=this.announcing?this.announcement:``;return i`
      <dialog
        class="panel ${this.tone}"
        aria-labelledby="stage-heading"
        @keydown=${this.handleKeydown}
        @cancel=${this.handleCancel}
        @close=${this.handleClose}
      >
        <div class="stage">
          ${this.glyphTemplate}
          <h2 id="stage-heading" class="stage-heading" tabindex="-1">
            ${this.headingText}
          </h2>
        </div>
        ${this.bodyTemplate}
        <p class="visually-hidden live-region" role="status">${e}</p>
      </dialog>
    `}updated(){let{dialog:e}=this;e&&e.isConnected&&!e.open&&(e.showModal(),this.focusHeading())}get hasRun(){let{stage:e}=this.runState;return e!==`idle`&&e!==`identifierTaken`}get glyphTemplate(){return this.reducedMotion&&this.tone===`working`?i`<span class="glyph still-glyph" aria-hidden="true"
        >${te}</span
      >`:i`<ia-status-indicator
      class="glyph"
      aria-hidden="true"
      mode=${this.glyphMode}
    ></ia-status-indicator>`}get tone(){switch(this.runState.stage){case`transferFailed`:case`taskFailed`:return`problem`;case`ready`:case`statusUnavailable`:return`done`;default:return`working`}}get glyphMode(){return{working:`loading`,problem:`error`,done:`success`}[this.tone]}get headingText(){switch(this.runState.stage){case`checkingIdentifier`:return a(`Getting ready to upload…`);case`uploading`:return a(`Uploading your files…`);case`transferFailed`:return this.runState.failedDuring===`identifierCheck`?a(`Couldn't confirm the Page URL`):a(`Uploading stopped`);case`processing`:return a(`Processing your item…`);case`taskFailed`:return a(`A processing task failed`);case`ready`:return a(`Your item is ready!`);case`statusUnavailable`:return a(`Your files were sent`);default:return``}}get bodyTemplate(){let{runState:e}=this;switch(e.stage){case`checkingIdentifier`:return i`<p class="detail">
          ${a(`Checking that the Page URL is still free…`)}
        </p>`;case`uploading`:{let{fileIndex:t,fileCount:n}=e,i=t+1,o=this.fileName(t);return this.progressTemplate(a(r`Uploading file ${i} of ${n}: ${o}`),e.bytesSent,e.bytesTotal)}case`transferFailed`:return this.transferFailedTemplate(e);case`processing`:return i`
          <p class="detail task-state">${e.taskState===`running`?a(`Processing…`):a(`Waiting to be processed…`)}</p>
          <horizontal-activity-indicator
            class="activity"
            aria-hidden="true"
            .adjustSpeedForWidth=${!1}
          ></horizontal-activity-indicator>
          <p class="detail">
            ${a(`All your files were sent. Processing carries on if you leave this page.`)}
          </p>
        `;case`taskFailed`:return i`
          <p class="detail">
            ${a(`Your files were sent, but a processing task for this item failed. The item's page shows what happened.`)}
          </p>
          <div class="actions">
            <a class="item-link" href=${E(e.identifier)}>
              ${a(`Go to your item`)}
            </a>
            <button
              type="button"
              class="secondary close"
              @click=${this.handleCloseClick}
            >
              ${a(`Close`)}
            </button>
          </div>
        `;case`ready`:return this.readyTemplate(e.identifier);case`statusUnavailable`:return i`
          <p class="detail">
            ${a(`We can't check on processing from here right now, but it carries on without this page. Your item's page shows its progress.`)}
          </p>
          <div class="actions">
            <a class="item-link" href=${E(e.identifier)}>
              ${a(`Go to your item`)}
            </a>
            <button
              type="button"
              class="secondary upload-another"
              @click=${this.handleUploadAnotherClick}
            >
              ${a(`Upload another item`)}
            </button>
          </div>
        `;default:return n}}readyTemplate(e){let t=E(e);return this.redirectCancelled?i`
        <p class="detail">${a(`You'll stay on this page.`)}</p>
        <div class="actions">
          <a class="item-link" href=${t}>${a(`Go to your item`)}</a>
          <button
            type="button"
            class="secondary upload-another"
            @click=${this.handleUploadAnotherClick}
          >
            ${a(`Upload another item`)}
          </button>
        </div>
      `:i`
      <p class="detail">
        ${a(r`Taking you to your item in ${10} seconds.`)}
      </p>
      <div class="actions">
        <a class="item-link" href=${t}>${a(`Go to your item now`)}</a>
        <button
          type="button"
          class="secondary stay"
          @click=${this.handleStayClick}
        >
          ${a(`Stay on this page`)}
        </button>
      </div>
    `}transferFailedTemplate(e){let t=Tt(e),n=Et(e);if(e.failedDuring===`identifierCheck`)return i`
        <p class="detail failure">${t}</p>
        <p class="detail">${n}</p>
        ${this.failureActionsTemplate(e)}
      `;let{fileIndex:o,fileCount:s}=e,c=o+1,l=this.fileName(o),u=this.files.slice(0,o).reduce((e,t)=>e+t.file.size,0),d=this.files.reduce((e,t)=>e+t.file.size,0);return i`
      ${this.progressTemplate(a(r`Stopped at file ${c} of ${s}: ${l}`),u,d)}
      <p class="detail failure">
        ${a(r`${l} couldn't be sent.`)} ${t}
      </p>
      <p class="detail">${n}</p>
      ${this.failureActionsTemplate(e)}
    `}failureActionsTemplate(e){let t=Gr(e),r=t.retry===`tryAgain`?a(`Try again`):a(`Resume uploading`);return i`
      <div class="actions">
        ${t.retry?i`<button
          type="button"
          class="resume"
          @click=${this.handleResumeClick}
        >
          ${r}
        </button>`:n} ${t.backToForm?i`<button
          type="button"
          class="secondary back-to-form"
          @click=${this.handleBackToFormClick}
        >
          ${a(`Back to the form`)}
        </button>`:n} ${t.classicUploader?i`<a class="classic-uploader-link" href=${this.classicUploaderUrl}
          >${a(`Use the classic uploader`)}</a
        >`:n} ${t.itemLink?i`<a class="item-link" href=${E(e.identifier)}
          >${a(`Go to your item`)}</a
        >`:n}
      </div>
    `}progressTemplate(e,t,n){let o=C(t),s=C(n);return i`
      <div class="progress">
        <label class="progress-label" for="overall-progress">${e}</label>
        <progress
          id="overall-progress"
          max=${Math.max(n,1)}
          .value=${t}
          aria-describedby="progress-bytes"
        ></progress>
        <p class="progress-bytes" id="progress-bytes">
          ${a(r`${o} of ${s} sent`)}
        </p>
      </div>
    `}fileName(e){return this.files[e]?.path??``}handleResumeClick(){this.dispatchEvent(D(`resumeRequested`,{}))}handleBackToFormClick(){this.dispatchEvent(D(`backToFormRequested`,{}))}handleStayClick(){this.dispatchEvent(D(`stayOnPageRequested`,{}))}handleCloseClick(){this.dispatchEvent(D(`closeRequested`,{}))}handleUploadAnotherClick(){this.dispatchEvent(D(`uploadAnotherRequested`,{}))}handleKeydown(e){e.key!==`Escape`||e.defaultPrevented||(e.preventDefault(),this.handleEscape())}handleCancel(e){e.preventDefault(),this.handleEscape()}handleClose(){this.hasRun&&this.requestUpdate()}handleEscape(){switch(Wr(this.runState)){case`backToForm`:this.handleBackToFormClick();break;case`close`:this.handleCloseClick();break;default:break}}static get styles(){return e`
      :host {
        display: contents;
        color: var(--ia-theme-primary-text-color, #2c2c2c);
      }

      :host([hidden]) {
        display: none;
      }

      /* The card, centred in the top layer over a dimmed page */
      .panel {
        --tone-color: var(--ia-theme-link-color, #4b64ff);
        box-sizing: border-box;
        width: calc(100% - 64px);
        max-width: 560px;
        max-height: calc(100% - 64px);
        margin: auto;
        padding: 22px 24px 24px;
        overflow: auto;
        background: var(--ia-theme-secondary-background-color, #fff);
        color: var(--ia-theme-primary-text-color, #2c2c2c);
        border: 1px solid var(--ia-theme-border-color, #ccc);
        border-left: 4px solid var(--tone-color);
        border-radius: 6px;
        box-shadow: 0 12px 32px rgb(0 0 0 / 30%);
        font-family: var(
          --ia-theme-base-font-family,
          'Helvetica Neue',
          Helvetica,
          Arial,
          sans-serif
        );
        font-size: 1.4rem;
        line-height: 1.43;
      }

      /* Dark enough to set the card apart in both themes */
      .panel::backdrop {
        background: rgb(0 0 0 / 55%);
      }

      /* Clicking the card's background focuses the dialog itself, which
         isn't a control, so it shows no ring */
      .panel:focus {
        outline: none;
      }

      .panel.problem {
        --tone-color: var(--ia-theme-color-danger-text, #c62828);
      }

      .panel.done {
        --tone-color: var(--ia-theme-color-success-text, #267326);
      }

      .stage {
        display: flex;
        align-items: center;
        gap: 12px;
      }

      .glyph {
        --icon-width: 28px;
        --primary-text-color: var(--tone-color);
        --color-success: var(--tone-color);
        --color-danger: var(--tone-color);
        display: block;
        flex: none;
        width: 28px;
        height: 28px;
      }

      .still-glyph {
        color: var(--tone-color);
      }

      .still-glyph svg {
        display: block;
        width: 100%;
        height: 100%;
      }

      .stage-heading {
        margin: 0;
        font-size: 1.8rem;
        line-height: 1.3;
        font-weight: 700;
      }

      .problem .stage-heading,
      .done .stage-heading {
        color: var(--tone-color);
      }

      .stage-heading:focus {
        outline: none;
      }

      .stage-heading:focus-visible {
        outline: 2px solid var(--ia-theme-link-color, #4b64ff);
        outline-offset: 2px;
      }

      .detail {
        margin: 12px 0 0;
        font-size: 1.4rem;
        line-height: 1.43;
      }

      .failure {
        font-weight: 700;
      }

      .progress {
        display: grid;
        gap: 6px;
        margin-top: 14px;
      }

      .progress-label {
        font-size: 1.4rem;
        font-weight: 700;
        overflow-wrap: anywhere;
      }

      progress {
        width: 100%;
        height: 12px;
        accent-color: var(--tone-color);
      }

      .progress-bytes {
        margin: 0;
        font-size: 1.3rem;
        color: var(--ia-theme-secondary-text-color, #666);
        font-variant-numeric: tabular-nums;
      }

      .activity {
        --horizontalActivityIndicatorColor: var(--tone-color);
        height: 4px;
        margin-top: 10px;
        overflow: hidden;
        border-radius: 2px;
        background: var(--ia-theme-border-color, #ccc);
      }

      .actions {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 12px 16px;
        margin-top: 16px;
      }

      .resume,
      .secondary {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        height: 40px;
        padding: 0 18px;
        border-radius: 4px;
        font: inherit;
        font-size: 1.5rem;
        font-weight: 700;
        cursor: pointer;
      }

      .resume {
        border: 1px solid var(--upload-primary-button-fill, #4b64ff);
        background: var(--upload-primary-button-fill, #4b64ff);
        color: #fff;
      }

      .resume:hover {
        background: var(--upload-primary-button-hover-fill, #3b52e6);
      }

      /* The white outline button, in both themes: #194880 text needs a
         light surface to meet contrast. */
      .secondary {
        border: 1px solid var(--ia-theme-primary-cta-fill, #194880);
        background: #fff;
        color: var(--ia-theme-primary-cta-fill, #194880);
      }

      .secondary:hover {
        background: #edf1ff;
      }

      .item-link,
      .classic-uploader-link {
        color: var(--ia-theme-link-color, #4b64ff);
        font-size: 1.4rem;
        font-weight: 700;
      }

      .resume:focus-visible,
      .secondary:focus-visible,
      .item-link:focus-visible,
      .classic-uploader-link:focus-visible {
        outline: 2px solid var(--ia-theme-link-color, #4b64ff);
        outline-offset: 2px;
      }

      .visually-hidden {
        position: absolute;
        clip: rect(0, 0, 0, 0);
        width: 1px;
        height: 1px;
        overflow: hidden;
        white-space: nowrap;
      }

      @media (max-width: 767px) {
        .panel {
          width: calc(100% - 32px);
          max-height: calc(100% - 32px);
          padding: 18px 16px 20px;
        }
      }

      @media (prefers-reduced-motion: reduce) {
        .activity {
          display: none;
        }
      }
    `}};_([t({type:Object})],X.prototype,`runState`,void 0),_([t({type:Array})],X.prototype,`files`,void 0),_([t({type:String})],X.prototype,`classicUploaderUrl`,void 0),_([t({type:Boolean})],X.prototype,`redirectCancelled`,void 0),_([t({type:Boolean})],X.prototype,`reducedMotion`,void 0),_([t({type:String})],X.prototype,`announcement`,void 0),_([m(`dialog`)],X.prototype,`dialog`,void 0),_([m(`.stage-heading`)],X.prototype,`stageHeading`,void 0),X=_([p(`upload-progress-panel`)],X);var Kr;function qr(e){return{shown:e,plain:e}}var Z=(Kr=class extends s{constructor(...e){super(...e),this.hasFiles=!1,this.missing=[],this.otherProblems=[]}get isUsable(){return this.hasFiles&&this.missing.length===0&&this.otherProblems.length===0}render(){let{isUsable:e}=this,t=this.helperSentences,r=a(` `,{desc:`Separates the sentences of the upload button's helper text.`}),o=t.map((e,t)=>i`${t>0?r:n}${e.shown}`),s=t.map(e=>e.plain).join(r);return i`
      <div class="bar">
        <button
          type="button"
          class=${e?`upload usable`:`upload`}
          aria-disabled=${f(e?void 0:`true`)}
          aria-describedby="helper-text"
          @click=${this.handleClick}
        >
          ${a(`Upload and Create Your Item`)}
        </button>
        <p class="helper" id="helper">${o}</p>
        <span id="helper-text" hidden>${s}</span>
      </div>
    `}get helperSentences(){if(!this.hasFiles)return[qr(a(`Add at least one file to begin.`))];if(this.isUsable)return[qr(a(`Ready to upload.`))];let e=[];if(this.missing.length>0){let t=this.missing.length,o=w.length,s=a(`, `,{desc:`Separates the field names in "2 of 5 required fields remaining — Description, Subject Tags."`}),c=this.missing.map((e,t)=>i`${t>0?s:n}${this.fieldButton(e,z(e))}`),l=this.missing.map(e=>z(e)).join(s);e.push({shown:a(i`${t} of ${o} required fields remaining — ${c}.`),plain:a(r`${t} of ${o} required fields remaining — ${l}.`)})}if(this.otherProblems.includes(`date`)){let t=a(`Date`);e.push({shown:a(i`Fix the ${this.fieldButton(`date`,t)}.`),plain:a(r`Fix the ${t}.`)})}if(this.otherProblems.includes(`custom`)){let t=a(`custom metadata`);e.push({shown:a(i`Fix the ${this.fieldButton(`custom`,t)}.`),plain:a(r`Fix the ${t}.`)})}return e}fieldButton(e,t){return i`<button
      type="button"
      class="field-link"
      data-field=${e}
      @click=${()=>this.requestFieldFocus(e)}
    >${t}</button>`}handleClick(e){if(!this.isUsable){e.preventDefault();return}this.dispatchEvent(D(`uploadRequested`,{}))}requestFieldFocus(e){this.dispatchEvent(D(`fieldFocusRequested`,{field:e}))}static get styles(){return e`
      :host {
        display: block;
        color: var(--ia-theme-primary-text-color, #333);
        font-family: var(
          --ia-theme-base-font-family,
          'Helvetica Neue',
          Helvetica,
          Arial,
          sans-serif
        );
        font-size: 14px;
        line-height: 1.42857;
      }

      :focus-visible {
        outline: 2px solid var(--ia-theme-link-color, #4b64ff);
        outline-offset: 2px;
      }

      .bar {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 10px;
        padding-top: 26px;
        border-top: 1px solid var(--ia-theme-border-color, #d3d3d3);
      }

      .upload {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        height: 50px;
        padding: 0 30px;
        border-radius: 4px;
        font: inherit;
        font-size: 17px;
        font-weight: 700;
        line-height: 1;
        white-space: nowrap;
      }

      /* Inactive: grey, dashed and still focusable (aria-disabled). Theme
         tokens keep it readable in the dark theme: the panel fill, the
         field border and the muted text. */
      .upload:not(.usable) {
        border: 1px dashed var(--ia-theme-field-border-color, #767676);
        background: var(--ia-theme-tertiary-background-color, #e6e6e6);
        color: var(--ia-theme-secondary-text-color, #555);
        cursor: not-allowed;
      }

      /* #4b64ff with white text in both themes: the dark theme's link
         colour is too light to sit behind white text. */
      .upload.usable {
        border: 1px solid var(--upload-primary-button-border, #3b52e6);
        background: var(--upload-primary-button-fill, #4b64ff);
        color: #fff;
        cursor: pointer;
      }

      .upload.usable:hover {
        background: var(--upload-primary-button-hover-fill, #3b52e6);
      }

      .helper {
        margin: 0;
        color: var(--ia-theme-secondary-text-color, #595959);
        font-size: 14px;
        text-align: center;
      }

      .field-link {
        padding: 0 1px;
        border: 0;
        border-radius: 2px;
        background: none;
        color: var(--ia-theme-link-color, #4b64ff);
        font: inherit;
        font-weight: 700;
        text-decoration: underline;
        text-underline-offset: 2px;
        cursor: pointer;
      }

      @media (max-width: 767px) {
        .upload {
          width: 100%;
        }
      }
    `}},Kr.shadowRootOptions={...s.shadowRootOptions,delegatesFocus:!0},Kr);_([t({type:Boolean})],Z.prototype,`hasFiles`,void 0),_([t({attribute:!1})],Z.prototype,`missing`,void 0),_([t({attribute:!1})],Z.prototype,`otherProblems`,void 0),Z=_([p(`upload-submit-bar`)],Z);var Jr=500,Q=i`<svg
  class="warning-glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <path
    d="M8 1.3 15.3 14.4H.7Z"
    fill="currentColor"
    stroke="currentColor"
    stroke-width=".8"
    stroke-linejoin="round"
  />
  <path
    d="M8 6.1v3.7"
    stroke="#fff"
    stroke-width="1.8"
    stroke-linecap="round"
  />
  <circle cx="8" cy="12" r="1" fill="#fff" />
</svg>`,Yr=i`<svg
  class="info-glyph"
  viewBox="0 0 16 16"
  aria-hidden="true"
  focusable="false"
>
  <circle
    cx="8"
    cy="8"
    r="6.9"
    fill="none"
    stroke="currentColor"
    stroke-width="1.4"
  />
  <path
    d="M8 7.2v4.3"
    stroke="currentColor"
    stroke-width="1.6"
    stroke-linecap="round"
  />
  <circle cx="8" cy="4.8" r="1" fill="currentColor" />
</svg>`,Xr=`/create.php?lma=1`,Zr=`https://blog.archive.org/2013/02/08/presetting-metadata-with-the-new-beta-uploader/`,$=class extends s{constructor(...e){super(...e),this.contextLoad={state:`loading`},this.fileList=Ce.empty(),this.skipped=[],this.fileAnnouncement=``,this.pickFailed=!1,this.filesReading=!1,this.uploadWaitsForFiles=!1,this.filesIgnoredDuringRun=!1,this.metadata=mt(),this.identifierStatus={state:`idle`},this.runState={stage:`idle`},this.runFiles=[],this.redirectCancelled=!1,this.progressDialogClosed=!1,this.reducedMotion=!1,this.presets={metadata:{},extraRows:[]},this.collectionEdited=!1,this.identifierEdited=!1,this.proposalNumber=0,this.warnsBeforeUnload=!1,this.customRowCount=0,this.setupStarted=!1,this.handleDocumentDragOver=e=>{Qe(e)&&(!e.defaultPrevented&&e.dataTransfer&&(e.dataTransfer.dropEffect=`none`),e.preventDefault())},this.handleDocumentDrop=e=>{Qe(e)&&e.preventDefault()},this.searchCollections=e=>{let{uploadService:t}=this;return t?t.searchCollections(e):Promise.reject(Error(`The upload service is not ready`))},this.handleReducedMotionChange=e=>{this.reducedMotion=e.matches},this.handleBeforeUnload=e=>{e.preventDefault(),e.returnValue=!0}}render(){return i`
      <div class="page">
        ${this.headerTemplate} ${this.contentTemplate}
        <p class="visually-hidden live-region" role="status">
          ${this.liveAnnouncement}
        </p>
      </div>
    `}updated(e){e.has(`appServices`)&&this.setupAppServicesDependents()}connectedCallback(){super.connectedCallback(),document.addEventListener(`dragover`,this.handleDocumentDragOver),document.addEventListener(`drop`,this.handleDocumentDrop),this.reducedMotionQuery=window.matchMedia(`(prefers-reduced-motion: reduce)`),this.reducedMotion=this.reducedMotionQuery.matches,this.reducedMotionQuery.addEventListener(`change`,this.handleReducedMotionChange)}disconnectedCallback(){super.disconnectedCallback(),document.removeEventListener(`dragover`,this.handleDocumentDragOver),document.removeEventListener(`drop`,this.handleDocumentDrop),this.reducedMotionQuery?.removeEventListener(`change`,this.handleReducedMotionChange),this.run?.cancel(),this.checker?.dispose(),this.clearProposalTimer(),clearTimeout(this.redirectTimer),this.redirectTimer=void 0,this.setWarnsBeforeUnload(!1)}navigateToUrl(e){window.location.href=e}get currentSearch(){return this.locationSearch??window.location.search}get classicUploaderUrl(){return Ze(this.currentSearch)}get hasFiles(){return this.fileList.totals.fileCount>0}get isRunning(){return this.run!==void 0}get formLockedMessage(){let{stage:e}=this.runState;return a(e===`ready`||e===`taskFailed`||e===`statusUnavailable`?`Locked because your item has been created`:`Locked while uploading`)}get liveAnnouncement(){return this.isRunning?``:this.uploadWaitsForFiles?this.waitingForFilesText:``}get runAnnouncementText(){return Ot(this.runState,this.runFiles,{redirectCancelled:this.redirectCancelled})}get waitingForFilesText(){return a(`Still adding the dropped files. You can upload once they're in the list.`)}get headerTemplate(){return i`
      <header class="page-header">
        <h1>${a(`Upload an Archive Item`)}</h1>
        <p class="intro">
          ${a(`Please contribute books, audio, and video files that you have the right to share. The Internet Archive, a non-profit library, will provide free storage and access to them.`)}
          ${a(`We reserve the right to remove any submitted material.`)}
        </p>
        <div class="header-links">
          <p class="lma-callout">
            ${Yr}
            <span>
              <strong>${a(`Contributing to the Live Music Archive?`)}</strong>
              <a href=${Xr}
                >${a(`Click here to use the LMA uploader instead.`)}</a
              >
            </span>
          </p>
        </div>
      </header>
    `}get footerTemplate(){return i`
      <p class="page-footer">
        <a href=${Zr} target="_blank" rel="noopener">
          ${a(`Instructions on how to preset metadata`)}
          <span class="visually-hidden">${a(`(opens in a new tab)`)}</span>
        </a>
      </p>
    `}get contentTemplate(){let{contextLoad:e}=this;if(e.state===`ready`)return this.formTemplate(e.context);let t=e.state===`loading`,n=this.reducedMotion?i`<span class="loading-indicator still-glyph" aria-hidden="true"
          >${te}</span
        >`:i`<ia-status-indicator
          class="loading-indicator"
          aria-hidden="true"
          mode="loading"
        ></ia-status-indicator>`,r=t?i`
          ${n}
          <p>${a(`Loading the uploader…`)}</p>
        `:i`
          <p class="unavailable-message">
            ${Q}
            <strong
              >${a(`The new uploader isn't available right now.`)}</strong
            >
          </p>
          <p>
            <a class="unavailable-link" href=${this.classicUploaderUrl}
              >${a(`Use the classic uploader`)}</a
            >
          </p>
        `;return i`
      <div
        class="load-state ${t?`loading`:`unavailable`}"
        role="status"
      >
        ${r}
      </div>
    `}formTemplate(e){let{hasFiles:t,isRunning:n,metadata:r,identifierStatus:a}=this,o=dt({metadata:r,hasFiles:t,identifierStatus:a}),s=$e(this.runState,this.runFiles),c=this.checker?.maxLength??80,l=yt({largestFile:this.fileList.totals.largestFile,context:e}),u=n?this.progressDialogTemplate:i`${this.waitingForFilesTemplate}
          <upload-submit-bar
            ?hasFiles=${t}
            .missing=${o.missing}
            .otherProblems=${o.otherProblems}
            @uploadRequested=${this.handleUploadRequested}
            @fieldFocusRequested=${this.handleFieldFocusRequested}
          ></upload-submit-bar>`;return i`
      ${this.takenNoticeTemplate} ${this.createdNoticeTemplate}
      <div class="columns">
        <div class="files-column">
          ${this.pickFailedTemplate} ${this.filesIgnoredTemplate}
          <upload-file-column
            .fileList=${this.fileList}
            .skipped=${this.skipped}
            .announcement=${this.fileAnnouncement}
            ?disabled=${n}
            .transferStatus=${s}
            @filesPicked=${this.handleFilesPicked}
            @filesPickFailed=${this.handleFilesPickFailed}
            @filesReadingChanged=${this.handleFilesReadingChanged}
            @fileNodeRemoved=${this.handleFileNodeRemoved}
          ></upload-file-column>
        </div>
        <div class="form-column">
          <upload-metadata-form
            .metadata=${r}
            .context=${e}
            ?hasFiles=${t}
            .fieldStates=${o.fieldStates}
            .identifierStatus=${a}
            .identifierMaxLength=${c}
            .dateError=${o.dateError}
            .customErrors=${o.customErrors}
            .searchCollections=${this.searchCollections}
            suggestedCollection=${l}
            ?disabled=${n}
            lockedMessage=${this.formLockedMessage}
            @metadataChanged=${this.handleMetadataChanged}
            @identifierSuggestionChosen=${this.handleIdentifierSuggestionChosen}
          ></upload-metadata-form>
        </div>
      </div>
      ${u} ${this.footerTemplate}
    `}get progressDialogTemplate(){return this.progressDialogClosed?n:i`<upload-progress-panel
      .runState=${this.runState}
      .files=${this.runFiles}
      .announcement=${this.runAnnouncementText}
      classicUploaderUrl=${this.classicUploaderUrl}
      ?redirectCancelled=${this.redirectCancelled}
      ?reducedMotion=${this.reducedMotion}
      @resumeRequested=${this.handleResumeRequested}
      @backToFormRequested=${this.handleBackToFormRequested}
      @stayOnPageRequested=${this.handleStayOnPageRequested}
      @closeRequested=${this.handleCloseRequested}
      @uploadAnotherRequested=${this.handleUploadAnotherRequested}
    ></upload-progress-panel>`}get createdNoticeTemplate(){let{runState:e}=this;return this.progressDialogClosed&&e.stage===`taskFailed`?i`
      <div class="notice created-notice" tabindex="-1">
        ${Q}
        <p>
          ${a(`Your item was created, but a processing task failed.`)}
          <a href=${E(e.identifier)}
            >${a(`Go to your item`)}</a
          >
        </p>
      </div>
    `:n}get takenNoticeTemplate(){let e=this.takenIdentifier;return e===void 0?n:i`
      <p class="notice taken-notice">
        ${Q}
        <span>
          ${a(r`Someone else just took the Page URL ${e}, so nothing was uploaded. Pick another Page URL, then upload again. Everything else you entered is still here.`)}
        </span>
      </p>
    `}get pickFailedTemplate(){return this.pickFailed?i`
      <p class="notice pick-failed" role="alert">
        ${Q}
        <span>
          ${a(`A dropped file or folder couldn't be read, so nothing from it was added. Try again, or use "Add files or folders".`)}
        </span>
      </p>
    `:n}get filesIgnoredTemplate(){return this.filesIgnoredDuringRun?i`
      <p class="notice files-ignored" role="alert">
        ${Q}
        <span>${a(`Files added during an upload weren't included.`)}</span>
      </p>
    `:n}get waitingForFilesTemplate(){return this.uploadWaitsForFiles?i`<p class="waiting-for-files">${this.waitingForFilesText}</p>`:n}handleFilesPicked(e){if(this.isRunning){this.filesIgnoredDuringRun=!0;return}let t=this.fileList.add(e.detail.candidates);this.fileList=t.list,this.skipped=t.skipped,this.fileAnnouncement=ae(t),this.pickFailed=!1,this.filesIgnoredDuringRun=!1,this.applyFilePrefill()}handleFilesPickFailed(){this.pickFailed=!0}handleFilesReadingChanged(e){this.filesReading=e.detail.reading,this.filesReading||(this.uploadWaitsForFiles=!1)}handleFileNodeRemoved(e){if(this.isRunning)return;let t=this.fileList.remove(e.detail.nodeId);t.removed.length!==0&&(this.fileList=t.list,this.fileAnnouncement=oe(t),this.applyFilePrefill())}applyFilePrefill(){if(this.contextLoad.state!==`ready`)return;let{changes:e,titleSuggestion:t}=vt({metadata:this.metadata,largestFile:this.fileList.totals.largestFile,titleSuggestion:this.titleSuggestion,collectionEdited:this.collectionEdited,context:this.contextLoad.context,presetCollection:this.presets.presetCollection});this.titleSuggestion=t,Object.keys(e).length>0&&(this.metadata={...this.metadata,...e}),this.proposeIdentifier()}handleMetadataChanged(e){if(this.isRunning)return;let{changes:t}=e.detail,n=this.metadata.identifier;this.metadata={...this.metadata,...t},t.title!==void 0&&(this.titleSuggestion=void 0,this.scheduleTitleProposal()),t.collection!==void 0&&(this.collectionEdited=!0),t.identifier!==void 0&&(!this.identifierEdited&&this.identifierStatus.state===`error`&&t.identifier===n?(this.lastProposal=void 0,this.proposeIdentifier()):this.userSetIdentifier(t.identifier,{immediate:!1}))}handleIdentifierSuggestionChosen(e){if(this.isRunning)return;let{identifier:t}=e.detail;this.metadata={...this.metadata,identifier:t},this.userSetIdentifier(t,{immediate:!0})}userSetIdentifier(e,t){this.identifierEdited=e!==``,this.takenIdentifier=void 0,this.lastProposal=void 0,this.clearProposalTimer(),this.checker?.edit(e,{immediate:t.immediate})}proposeIdentifier(){this.clearProposalTimer();let{checker:e}=this;if(!(e!==void 0&&this.hasFiles&&!this.identifierEdited&&!this.isRunning))return;let t=(this.presets.presetSuggestedIdentifier??this.metadata.title).trim();if(!(t!==``&&t!==this.lastProposal))return;this.lastProposal=t,this.proposalNumber+=1;let n=this.proposalNumber;e.propose(t).then(t=>{n!==this.proposalNumber||this.identifierEdited||this.isRunning||t===void 0&&(this.lastProposal=void 0,e.status.state===`error`)||(t!==void 0&&t!==this.takenIdentifier&&(this.takenIdentifier=void 0),this.metadata={...this.metadata,identifier:t??``})})}scheduleTitleProposal(){this.presets.presetSuggestedIdentifier===void 0&&(this.clearProposalTimer(),this.proposalTimer=setTimeout(()=>{this.proposalTimer=void 0,this.proposeIdentifier()},Jr))}clearProposalTimer(){clearTimeout(this.proposalTimer),this.proposalTimer=void 0}handleFieldFocusRequested(e){this.metadataForm?.focusField(e.detail.field)}async handleUploadRequested(){let{uploadService:e,transferService:t,metadata:n}=this;if(this.isRunning||!e||!t)return;if(this.filesReading){this.uploadWaitsForFiles=!0;return}let{canUpload:r}=dt({metadata:n,hasFiles:this.hasFiles,identifierStatus:this.identifierStatus});if(!r)return;this.clearProposalTimer(),this.takenIdentifier=void 0,this.redirectCancelled=!1,this.progressDialogClosed=!1,this.filesIgnoredDuringRun=!1;let i=[...this.fileList.files],a=this.fileList.totals.largestFile,o=a?ze(a.file):void 0,s=new Kt({identifier:n.identifier,files:i,metadataHeaders:je(n,{inferredMediatype:o}),uploadService:e,transferService:t,onStateChange:e=>this.handleRunStateChange(s,e),maxIdentifierLength:this.checker?.maxLength});this.run=s,this.runFiles=i,this.runState=s.state,s.start(),await this.updateComplete,await this.progressPanel?.updateComplete,this.progressPanel?.focusHeading()}async handleResumeRequested(){let{run:e}=this;e&&(e.resume(),await this.focusPanelHeading())}async handleBackToFormRequested(){let e=this.runState;if(!(e.stage===`transferFailed`&&e.nothingUploaded))return;this.endRun(),await this.updateComplete;let{missing:t,otherProblems:n}=dt({metadata:this.metadata,hasFiles:this.hasFiles,identifierStatus:this.identifierStatus}),r=t[0]??n[0];if(r){await this.metadataForm?.updateComplete,this.metadataForm?.focusField(r);return}await this.submitBar?.updateComplete,this.submitBar?.focus()}async handleStayOnPageRequested(){this.runState.stage!==`ready`||this.redirectCancelled||(clearTimeout(this.redirectTimer),this.redirectTimer=void 0,this.redirectCancelled=!0,await this.focusPanelHeading())}async handleCloseRequested(){this.runState.stage===`taskFailed`&&(this.progressDialogClosed=!0,await this.updateComplete,this.createdNotice?.focus())}handleUploadAnotherRequested(){let{stage:e}=this.runState;(e===`ready`&&this.redirectCancelled||e===`statusUnavailable`)&&this.navigateToUrl(At(this.currentSearch))}async focusPanelHeading(){await this.updateComplete,await this.progressPanel?.updateComplete,this.progressPanel?.focusHeading()}handleRunStateChange(e,t){e===this.run&&(this.runState=t,this.setWarnsBeforeUnload(bt(t)),t.stage===`identifierTaken`?this.returnToForm(t.identifier,t.suggestions):t.stage===`ready`&&this.scheduleRedirect(t.identifier))}async returnToForm(e,t){this.endRun(),this.checker?.markTaken(e,t),this.takenIdentifier=e,await this.updateComplete,await this.metadataForm?.updateComplete,this.metadataForm?.focusField(`identifier`)}endRun(){this.run?.cancel(),this.run=void 0,this.runFiles=[],this.runState={stage:`idle`},this.progressDialogClosed=!1,this.setWarnsBeforeUnload(!1)}scheduleRedirect(e){clearTimeout(this.redirectTimer),this.redirectTimer=setTimeout(()=>{this.redirectTimer=void 0,this.navigateToUrl(E(e))},10*1e3)}setWarnsBeforeUnload(e){e!==this.warnsBeforeUnload&&(this.warnsBeforeUnload=e,e?window.addEventListener(`beforeunload`,this.handleBeforeUnload):window.removeEventListener(`beforeunload`,this.handleBeforeUnload))}async setupAppServicesDependents(){if(this.setupStarted)return;let e=this.appServices;if(e){this.setupStarted=!0;try{let t=await e.get(),[,n,r]=await Promise.allSettled([this.setupPageMetadata(t),t.uploadService.get(),t.uploadTransferService.get()]);if(n.status===`rejected`||r.status===`rejected`){this.contextLoad={state:`unavailable`};return}this.uploadService=n.value,this.transferService=r.value;let i=await this.uploadService.getContext();await this.startForm(this.uploadService,i)}catch{this.contextLoad={state:`unavailable`}}}}async setupPageMetadata(e){(await e.pageMetadataService.get()).setTitle(a(`Upload an Archive Item`))}async startForm(e,t){let n=Ge(new URLSearchParams(this.currentSearch));this.presets=n,this.checker=new Le({check:(t,n)=>e.checkIdentifier(t,n),onChange:e=>{this.identifierStatus=e},maxIdLength:n.maxIdLength}),this.metadata=ht({presets:n,context:t,nextRowId:()=>this.nextCustomRowId()}),this.contextLoad={state:`ready`,context:t},n.explicitIdentifier&&(this.identifierEdited=!0,this.checker.edit(n.explicitIdentifier,{immediate:!0})),this.metadata.custom.length>0&&(await this.updateComplete,await this.metadataForm?.updateComplete,this.metadataForm?.openSection(`additional`))}nextCustomRowId(){return this.customRowCount+=1,`preset-row-${this.customRowCount}`}static get styles(){return e`
      :host {
        --upload-primary-button-fill: #4b64ff;
        --upload-primary-button-hover-fill: #3b52e6;
        display: block;
        background: var(--ia-theme-secondary-background-color, #fff);
        color: var(--ia-theme-primary-text-color, #2c2c2c);
        font-family: var(
          --ia-theme-base-font-family,
          'Helvetica Neue',
          Helvetica,
          Arial,
          sans-serif
        );
        font-size: 1.4rem;
        line-height: 1.43;
      }

      .page {
        box-sizing: border-box;
        max-width: 1050px;
        margin: 0 auto;
        padding: 28px 32px 44px;
        display: flex;
        flex-direction: column;
        gap: 22px;
      }

      .page-header {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
      }

      h1 {
        margin: 0;
        font-size: 30px;
        line-height: 1.1;
        font-weight: 500;
        text-align: center;
      }

      p {
        margin: 0;
      }

      .intro {
        max-width: 720px;
        color: var(--ia-theme-secondary-text-color, #666);
        font-size: 1.4rem;
        line-height: 1.5;
        text-align: center;
      }

      .header-links {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        justify-content: center;
        gap: 8px 20px;
      }

      /* The page background rather than the section panel's, so the link
         keeps its contrast (#4b64ff is 4.00:1 on #efefef) */
      .lma-callout {
        display: flex;
        align-items: flex-start;
        gap: 8px;
        padding: 5px 12px;
        border: 1px solid var(--ia-theme-border-color, #ccc);
        border-left: 3px solid var(--ia-theme-link-color, #4b64ff);
        border-radius: 3px;
        background: var(--ia-theme-secondary-background-color, #fff);
        font-size: 1.3rem;
        line-height: 1.5;
      }

      .lma-callout a {
        text-decoration: underline;
        text-underline-offset: 2px;
      }

      .info-glyph {
        flex: none;
        width: 16px;
        height: 16px;
        margin-top: 2px;
        color: var(--ia-theme-link-color, #4b64ff);
      }

      .page-footer {
        margin-top: -6px;
        font-size: 1.3rem;
        text-align: center;
      }

      a {
        color: var(--ia-theme-link-color, #4b64ff);
        border-radius: 2px;
      }

      a:focus-visible,
      .created-notice:focus-visible {
        outline: 2px solid var(--ia-theme-link-color, #4b64ff);
        outline-offset: 2px;
      }

      .columns {
        display: grid;
        grid-template-columns: 38fr 62fr;
        gap: 28px;
        align-items: start;
      }

      .files-column,
      .form-column {
        display: flex;
        flex-direction: column;
        gap: 12px;
        min-width: 0;
      }

      .load-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        padding: 48px 16px;
        text-align: center;
      }

      .load-state.loading {
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .loading-indicator {
        --icon-width: 40px;
        --primary-text-color: var(--ia-theme-link-color, #4b64ff);
        display: block;
        width: 40px;
        height: 40px;
      }

      .still-glyph {
        color: var(--ia-theme-link-color, #4b64ff);
      }

      .still-glyph svg {
        display: block;
        width: 100%;
        height: 100%;
      }

      /* The page background rather than the section panel's, so the link
         keeps its contrast (#4b64ff is 4.00:1 on #efefef) */
      .load-state.unavailable {
        max-width: 560px;
        margin: 0 auto;
        box-sizing: border-box;
        background: var(--ia-theme-secondary-background-color, #fff);
        border: 1px solid var(--ia-theme-border-color, #ccc);
        border-left: 4px solid var(--ia-theme-color-danger-text, #c62828);
        border-radius: 3px;
        padding: 20px 22px;
      }

      .unavailable-message {
        display: flex;
        align-items: center;
        gap: 8px;
        color: var(--ia-theme-color-danger-text, #c62828);
        font-size: 1.6rem;
      }

      .unavailable-link {
        font-weight: 700;
      }

      .notice {
        display: flex;
        align-items: flex-start;
        gap: 8px;
        padding: 10px 12px;
        border: 1px solid var(--ia-theme-color-danger-text, #c62828);
        border-radius: 3px;
        background: var(--ia-theme-secondary-background-color, #fff);
        color: var(--ia-theme-color-danger-text, #c62828);
        font-weight: 700;
      }

      .warning-glyph {
        flex: none;
        width: 16px;
        height: 16px;
        margin-top: 2px;
      }

      /* The item exists, so the words are in the ink colour: only the glyph
         and the edge say something went wrong */
      .created-notice {
        border-color: var(--ia-theme-border-color, #ccc);
        border-left: 4px solid var(--ia-theme-color-danger-text, #c62828);
        color: var(--ia-theme-primary-text-color, #2c2c2c);
        font-weight: 400;
      }

      .created-notice .warning-glyph {
        color: var(--ia-theme-color-danger-text, #c62828);
      }

      .created-notice a {
        font-weight: 700;
      }

      .waiting-for-files {
        color: var(--ia-theme-secondary-text-color, #666);
        text-align: center;
      }

      .visually-hidden {
        position: absolute;
        clip: rect(0, 0, 0, 0);
        width: 1px;
        height: 1px;
        overflow: hidden;
        white-space: nowrap;
      }

      @media (max-width: 767px) {
        .page {
          padding: 20px 16px 32px;
          gap: 18px;
        }

        h1 {
          font-size: 24px;
        }

        .header-links {
          flex-direction: column;
          gap: 8px;
        }

        .columns {
          grid-template-columns: minmax(0, 1fr);
          gap: 20px;
        }
      }
    `}};_([t({type:Object})],$.prototype,`appServices`,void 0),_([t({attribute:!1})],$.prototype,`locationSearch`,void 0),_([o()],$.prototype,`contextLoad`,void 0),_([o()],$.prototype,`fileList`,void 0),_([o()],$.prototype,`skipped`,void 0),_([o()],$.prototype,`fileAnnouncement`,void 0),_([o()],$.prototype,`pickFailed`,void 0),_([o()],$.prototype,`filesReading`,void 0),_([o()],$.prototype,`uploadWaitsForFiles`,void 0),_([o()],$.prototype,`filesIgnoredDuringRun`,void 0),_([o()],$.prototype,`metadata`,void 0),_([o()],$.prototype,`identifierStatus`,void 0),_([o()],$.prototype,`runState`,void 0),_([o()],$.prototype,`runFiles`,void 0),_([o()],$.prototype,`takenIdentifier`,void 0),_([o()],$.prototype,`redirectCancelled`,void 0),_([o()],$.prototype,`progressDialogClosed`,void 0),_([o()],$.prototype,`reducedMotion`,void 0),_([m(`upload-metadata-form`)],$.prototype,`metadataForm`,void 0),_([m(`upload-progress-panel`)],$.prototype,`progressPanel`,void 0),_([m(`upload-submit-bar`)],$.prototype,`submitBar`,void 0),_([m(`.created-notice`)],$.prototype,`createdNotice`,void 0),$=_([p(`upload-page`)],$);var Qr=$;export{Qr as default};