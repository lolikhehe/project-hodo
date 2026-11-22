import{A as e,C as t,E as n,F as r,N as i,O as a,P as o,S as s,T as c,_ as l,a as u,f as d,i as f,j as p,k as m,l as h,m as g,o as _,p as v,r as ee,s as y,u as te,v as b,w as x,x as S}from"./vendor-lit-MQ4DzmMc.js";import{i as ne,n as re,s as C}from"./vendor-shared-62wh2Xkx.js";import{r as ie,t as ae}from"./vendor-topnav-Dcg-INpV.js";import{i as oe}from"./dist-BqHmxVdQ.js";import"./ia-status-indicator-Bd61y7bd.js";import"./ia-clearable-text-input-CaSQalrP.js";import"./ia-dropdown-DbuqBiPJ.js";import"./dist-CTMeFT4b.js";import{a as w,n as se,r as ce,s as T}from"./dist-P4PnKJsG.js";import{t as le}from"./purify.es-RmFZhtLI.js";import"./dist-CD3AhsrO.js";function ue(e){return e?.replace(/%22%22(?!%22%22)(.+?)%22%22/g,`%22$1%22`)}function de(e){return e?.rawMetadata?.hit_type===`favorited_search`?`search`:e.mediatype?.value??`data`}var fe=m`
  <svg
    viewBox="0 0 100 100"
    xmlns="http://www.w3.org/2000/svg"
  >
    <g fill="#000" fill-rule="evenodd">
      <path
        d="m86 0c5.5228475 0 10 4.4771525 10 10v80c0 5.5228475-4.4771525 10-10 10h-57c-5.5228475 0-10-4.4771525-10-10v-13h7.999l.001 8c0 2.6887547 2.1223067 4.8818181 4.7831104 4.9953805l.2168896.0046195h50.06c2.6887547 0 4.8818181-2.1223067 4.9953805-4.7831104l.0046195-.2168896v-70c0-2.7614237-2.2385763-5-5-5h-50.06c-2.6887547 0-4.8818181 2.1223067-4.9953805 4.7831104l-.0046195.2168896-.001 7h-7.999v-12c0-5.5228475 4.4771525-10 10-10z"
      />
      <path
        d="m4 52h14c-.0377367 5.0230706-.0377367 15.3564039 0 31h35v-31h16c-20.0693886-21.0798163-30.8885707-32.4131497-32.4575462-34z" fill-rule="nonzero" transform="matrix(0 1 -1 0 87 14)"
      />
    </g>
    <title>Log in to view this item</title>
  </svg>
`,pe=m`
  <svg
    viewBox="0 0 100 100"
    xmlns="http://www.w3.org/2000/svg"
  >
    <path
      d="m56.4612493 8.80450354 41.8901185 75.94632926c1.7706782 2.8433173 2.1150372 5.2623412 1.0330766 7.2570716-1.0819604 1.9947304-3.26978 2.9920956-6.5634587 2.9920956h-85.69973905c-3.29367873 0-5.46954894-.9973652-6.52761065-2.9920956-1.0580617-1.9947304-.70175345-4.4137543 1.06892476-7.2570716l41.89011844-75.12308969c1.8184757-2.84331737 3.9693609-4.37738627 6.4526556-4.60220671s4.6341799 1.03483527 6.4526556 3.77896714zm28.5387507 75.19549646-35.037482-62-34.962518 62zm-31-34.7484359v-10.2515641h-8v10.2515641l2.089172 14.7484359h3.8184713zm-8 19.7484359v8h8v-8z"
    />
    <title>Content may be inappropriate</title>
  </svg>
`,me={get"login-required"(){return o(`Log in to view this item`)},get"content-warning"(){return o(`Content may be inappropriate`)}},he={"login-required":fe,"content-warning":pe},ge=class e{constructor(t,n=`unknown`){let r=this.getFlags(t);this.adIds=t.ad_id?.values,this.averageRating=t.avg_rating?.value,this.captureDates=t.capture_dates?.values,this.checked=!1,this.collections=t.collection?.values??[],this.collectionFilesCount=t.collection_files_count?.value??0,this.collectionSize=t.collection_size?.value??0,this.commentCount=t.num_reviews?.value??0,this.creator=t.creator?.value,this.creators=t.creator?.values??[],this.dateAdded=t.addeddate?.value,this.dateArchived=t.publicdate?.value,this.datePublished=t.date?.value,this.dateReviewed=t.reviewdate?.value,this.description=t.description?.values.join(`
`),this.factChecks=t.factcheck?.values,this.favCount=t.num_favorites?.value??0,this.hitRequestSource=n,this.hitType=t.rawMetadata?.hit_type,this.href=ue(t.review?.__href__??t.__href__?.value),this.identifier=e.cleanIdentifier(t.identifier),this.isClip=t.is_clip?.value,this.issue=t.issue?.value,this.itemCount=t.item_count?.value??0,this.mediatype=de(t),this.review=t.review,this.snippets=t.highlight?.values??[],this.source=t.source?.value,this.subjects=t.subject?.values??[],this.thumbnailUrl=t.__img__?.value,this.title=t.title?.value??``,this.tvClipCount=t.num_clips?.value??0,this.volume=t.volume?.value,this.viewCount=t.downloads?.value,this.weeklyViewCount=t.week?.value,this.loginRequired=r.loginRequired,this.contentWarning=r.contentWarning}clone(){let t=new e({});return t.adIds=this.adIds,t.averageRating=this.averageRating,t.captureDates=this.captureDates,t.checked=this.checked,t.collections=this.collections,t.collectionFilesCount=this.collectionFilesCount,t.collectionSize=this.collectionSize,t.commentCount=this.commentCount,t.creator=this.creator,t.creators=this.creators,t.dateStr=this.dateStr,t.dateAdded=this.dateAdded,t.dateArchived=this.dateArchived,t.datePublished=this.datePublished,t.dateReviewed=this.dateReviewed,t.description=this.description,t.factChecks=this.factChecks,t.favCount=this.favCount,t.hitRequestSource=this.hitRequestSource,t.hitType=this.hitType,t.href=this.href,t.identifier=this.identifier,t.isClip=this.isClip,t.issue=this.issue,t.itemCount=this.itemCount,t.mediatype=this.mediatype,t.snippets=this.snippets,t.source=this.source,t.subjects=this.subjects,t.thumbnailUrl=this.thumbnailUrl,t.title=this.title,t.tvClipCount=this.tvClipCount,t.volume=this.volume,t.viewCount=this.viewCount,t.weeklyViewCount=this.weeklyViewCount,t.loginRequired=this.loginRequired,t.contentWarning=this.contentWarning,t}get isTvSearchResult(){return this.hitType===`tv_clip`&&this.hitRequestSource===`search_query`}getFlags(e){let t={loginRequired:!1,contentWarning:!1};if(e.collection?.values.length&&e.mediatype?.value!==`collection`){for(let n of e.collection?.values??[])if(n===`loggedin`&&(t.loginRequired=!0,t.contentWarning)||n===`no-preview`&&(t.contentWarning=!0,t.loginRequired))break}return t}static cleanIdentifier(e){let t=e?.indexOf(`|`)??-1;return t>0?e?.slice(0,t):e}},E;(function(e){e.default=`default`,e.unrecognized=`unrecognized`,e.relevance=`relevance`,e.alltimeview=`alltimeview`,e.weeklyview=`weeklyview`,e.title=`title`,e.date=`date`,e.datearchived=`datearchived`,e.datereviewed=`datereviewed`,e.dateadded=`dateadded`,e.datefavorited=`datefavorited`,e.creator=`creator`})(E||(E={})),E.weeklyview,E.alltimeview,E.datefavorited,E.date,E.datearchived,E.datereviewed,E.dateadded;var D={[E.default]:{field:E.default,defaultSortDirection:null,canSetDirection:!1,shownInSortBar:!1,shownInURL:!1,handledBySearchService:!1,displayName:``,urlNames:[``,null,void 0]},[E.unrecognized]:{field:E.unrecognized,defaultSortDirection:null,canSetDirection:!0,shownInSortBar:!1,shownInURL:!1,handledBySearchService:!0,displayName:``,urlNames:[]},[E.relevance]:{field:E.relevance,defaultSortDirection:null,canSetDirection:!1,shownInSortBar:!0,shownInURL:!1,handledBySearchService:!1,get displayName(){return o(`Relevance`)},urlNames:[`_score`]},[E.alltimeview]:{field:E.alltimeview,defaultSortDirection:`desc`,canSetDirection:!0,shownInSortBar:!0,shownInURL:!0,handledBySearchService:!0,searchServiceKey:`downloads`,get displayName(){return o(`All-time views`)},urlNames:[`downloads`]},[E.weeklyview]:{field:E.weeklyview,defaultSortDirection:`desc`,canSetDirection:!0,shownInSortBar:!0,shownInURL:!0,handledBySearchService:!0,searchServiceKey:`week`,get displayName(){return o(`Weekly views`)},urlNames:[`week`]},[E.title]:{field:E.title,defaultSortDirection:`asc`,canSetDirection:!0,shownInSortBar:!0,shownInURL:!0,handledBySearchService:!0,searchServiceKey:`titleSorter`,get displayName(){return o(`Title`)},urlNames:[`title`,`titleSorter`]},[E.date]:{field:E.date,defaultSortDirection:`desc`,canSetDirection:!0,shownInSortBar:!0,shownInURL:!0,handledBySearchService:!0,searchServiceKey:`date`,get displayName(){return o(`Date published`)},urlNames:[`date`]},[E.datearchived]:{field:E.datearchived,defaultSortDirection:`desc`,canSetDirection:!0,shownInSortBar:!0,shownInURL:!0,handledBySearchService:!0,searchServiceKey:`publicdate`,get displayName(){return o(`Date archived`)},urlNames:[`publicdate`]},[E.datereviewed]:{field:E.datereviewed,defaultSortDirection:`desc`,canSetDirection:!0,shownInSortBar:!0,shownInURL:!0,handledBySearchService:!0,searchServiceKey:`reviewdate`,get displayName(){return o(`Date reviewed`)},urlNames:[`reviewdate`]},[E.dateadded]:{field:E.dateadded,defaultSortDirection:`desc`,canSetDirection:!0,shownInSortBar:!0,shownInURL:!0,handledBySearchService:!0,searchServiceKey:`addeddate`,get displayName(){return o(`Date added`)},urlNames:[`addeddate`]},[E.datefavorited]:{field:E.datefavorited,defaultSortDirection:`desc`,canSetDirection:!1,shownInSortBar:!0,shownInURL:!1,handledBySearchService:!1,searchServiceKey:`favoritedate`,get displayName(){return o(`Date favorited`)},urlNames:[`favoritedate`]},[E.creator]:{field:E.creator,defaultSortDirection:`asc`,canSetDirection:!0,shownInSortBar:!0,shownInURL:!0,handledBySearchService:!0,searchServiceKey:`creatorSorter`,get displayName(){return o(`Creator`)},urlNames:[`creator`,`creatorSorter`]}};function _e(e){return Object.values(D).find(t=>t.urlNames.some(t=>e===t))??D[E.unrecognized]}var ve={[E.relevance]:!0,[E.weeklyview]:!0,[E.alltimeview]:!0,[E.title]:!0,[E.datefavorited]:!1,[E.date]:!0,[E.datearchived]:!0,[E.datereviewed]:!0,[E.dateadded]:!0,[E.creator]:!0,[E.default]:!1,[E.unrecognized]:!1},ye={...ve,[E.datefavorited]:!0},be={...ve,[E.date]:!1,[E.datereviewed]:!1,[E.dateadded]:!1},xe={uploads:E.datearchived,reviews:E.datereviewed,collections:E.datearchived,web_archives:E.datearchived,favorites:E.datefavorited},Se={title:`firstTitle`,creator:`firstCreator`},O=()=>({subject:{},lending:{},mediatype:{},language:{},creator:{},collection:{},year:{},clip_type:{},program:{},person:{},sponsor:{}}),Ce={only_commercials:`commercial`,only_factchecks:`fact check`,only_quotes:`quote`},we=[`mediatype`,`year`,`subject`,`collection`,`creator`,`language`],Te=[`program`,`creator`,`year`,`subject`,`language`,`clip_type`],Ee={get subject(){return o(`Subject`)},get lending(){return o(`Availability`)},get mediatype(){return o(`Media Type`)},get language(){return o(`Language`)},get creator(){return o(`Creator`)},get collection(){return o(`Collection`)},get year(){return o(`Year`)},get clip_type(){return o(`Clip Type`)},get program(){return o(`Program`)},get person(){return o(`Person`)},get sponsor(){return o(`Sponsor`)}},De={get subject(){return o(`Subjects`)},get lending(){return o(`Availability`)},get mediatype(){return o(`Media Types`)},get language(){return o(`Languages`)},get creator(){return o(`Creators`)},get collection(){return o(`Collections`)},get year(){return o(`Years`)},get clip_type(){return o(`Clip Types`)},get program(){return o(`Programs`)},get person(){return o(`People`)},get sponsor(){return o(`Sponsors`)}},Oe={subject:T.COUNT,lending:T.COUNT,mediatype:T.COUNT,language:T.COUNT,creator:T.COUNT,collection:T.COUNT,year:T.NUMERIC,clip_type:T.COUNT,program:T.COUNT,person:T.COUNT,sponsor:T.COUNT},ke={...Oe,creator:T.ALPHABETICAL,program:T.ALPHABETICAL},Ae={subject:T.ALPHABETICAL,lending:T.ALPHABETICAL,mediatype:T.ALPHABETICAL,language:T.ALPHABETICAL,creator:T.ALPHABETICAL,collection:T.ALPHABETICAL,year:T.NUMERIC,clip_type:T.ALPHABETICAL,program:T.ALPHABETICAL,person:T.ALPHABETICAL,sponsor:T.ALPHABETICAL},je={is_lendable:!0,is_borrowable:!1,available_to_borrow:!0,is_browsable:!1,available_to_browse:!1,is_readable:!0,available_to_waitlist:!1},Me={lending:{get is_lendable(){return o(`Lending Library`)},get available_to_borrow(){return o(`Borrow 14 Days`)},get is_readable(){return o(`Always Available`)}},clip_type:{get quote(){return o(`Quote`)},get commercial(){return o(`Political Ad`)},get"fact check"(){return o(`Fact Check`)}}},Ne={deemphasize:!0,community:!0,stream_only:!0,samples_only:!0,test_collection:!0,printdisabled:!0,"openlibrary-ol":!0,nationalemergencylibrary:!0,china:!0,americana:!0,toronto:!0};function Pe(e,t){return e===t?!0:e.length===t.length?e.every((e,n)=>e===t[n]):!1}function Fe(e,t){return e?.includes(`-`)||t?.includes(`-`)?`date`:`year`}var Ie=`__proto__`,Le=class{constructor(e){this.cookieDomain=`.archive.org`,this.cookieExpiration=30,this.cookiePath=`/`,this.context=e.context}persistState(e,t={}){e.displayMode&&this.persistViewStateToCookies(e.displayMode),this.persistQueryStateToUrl(e,t)}getRestorationState(){let e=this.loadQueryStateFromUrl();return e.displayMode=this.loadTileViewStateFromCookies(),e}persistViewStateToCookies(e){let t=e===`grid`?`tiles`:`lists`;ne(`view-${this.context}`,t,{domain:this.cookieDomain,expires:this.cookieExpiration,path:this.cookiePath});let n=e===`list-detail`?`showdetails`:``;ne(`showdetails-${this.context}`,n,{domain:this.cookieDomain,expires:this.cookieExpiration,path:this.cookiePath})}loadTileViewStateFromCookies(){let e=re(`view-${this.context}`),t=re(`showdetails-${this.context}`);return e===`tiles`||e===void 0?`grid`:t===`showdetails`?`list-detail`:`list-compact`}persistQueryStateToUrl(e,t={}){let n=new URL(window.location.href),r=new URLSearchParams(n.searchParams),i=this.removeRecognizedParams(n.searchParams),a=!1;switch(e.baseQuery&&i.set(`query`,e.baseQuery),e.searchType){case w.FULLTEXT:i.set(`sin`,`TXT`);break;case w.RADIO:i.set(`sin`,`RADIO`);break;case w.TV:i.set(`sin`,`TV`);break;case w.METADATA:(t.persistMetadataSearchType||r.get(`sin`)===`MD`)&&i.set(`sin`,`MD`);break}if(r.get(`sin`)===``&&(r.delete(`sin`),a=!0),e.currentPage&&(e.currentPage>1?i.set(`page`,e.currentPage.toString()):i.delete(`page`)),e.selectedSort){let t=D[e.selectedSort],n=this.sortDirectionPrefix(e.sortDirection),a=e.searchType===w.TV&&e.selectedSort===E.relevance;if(t.field===E.unrecognized){let t=r.get(`sort`)??``,{field:a,direction:o}=this.getSortFieldAndDirection(t);e.sortDirection||(n=this.sortDirectionPrefix(o)),a?i.set(`sort`,`${n}${a}`):i.set(`sort`,t)}else if(t.shownInURL||a){let e=t.urlNames[0];i.set(`sort`,`${n}${e}`)}}if(e.selectedFacets)for(let[t,n]of Object.entries(e.selectedFacets)){let e=Object.entries(n);if(e.length!==0)for(let[n,r]of e){let e=r.state===`hidden`,a=`${t}:"${n}"`;e?i.append(`not[]`,a):i.append(`and[]`,a)}}let o=Fe(e.minSelectedDate,e.maxSelectedDate);e.minSelectedDate&&e.maxSelectedDate&&i.append(`and[]`,`${o}:[${e.minSelectedDate} TO ${e.maxSelectedDate}]`),e.titleQuery&&i.append(`and[]`,e.titleQuery),e.creatorQuery&&i.append(`and[]`,e.creatorQuery);let s=t.forceReplace?`replaceState`:`pushState`,c=this.paramsMatch(r,i,[`sin`,`sort`,`and[]`,`not[]`,`only_commercials`,`only_factchecks`,`only_quotes`]);if(c&&this.paramsMatch(r,i,[`query`])){if(a)i.delete(`sin`);else if(this.paramsMatch(r,i,[`page`]))return;s=`replaceState`}else c&&this.hasLegacyParam(r)&&(s=`replaceState`);window.history[s]?.({query:e.baseQuery,searchType:e.searchType,page:e.currentPage,sort:{field:e.selectedSort,direction:e.sortDirection},minDate:e.minSelectedDate,maxDate:e.maxSelectedDate,facets:e.selectedFacets},``,n)}loadQueryStateFromUrl(){let e=new URL(window.location.href),t=e.searchParams.get(`sin`),n=e.searchParams.get(`page`),r=e.searchParams.get(`query`),i=e.searchParams.get(`sort`),a=e.searchParams.getAll(`and[]`),o=e.searchParams.getAll(`not[]`);for(let[t,n]of e.searchParams.entries())/and\[\d+\]/.test(t)?a.push(n):/not\[\d+\]/.test(t)&&o.push(n);let s=e.searchParams.get(`q`)??e.searchParams.get(`search`),c={selectedFacets:O()};switch(r?c.baseQuery=r:s&&(c.baseQuery=s),t){case`TXT`:c.searchType=w.FULLTEXT;break;case`RADIO`:c.searchType=w.RADIO;break;case`TV`:c.searchType=w.TV;break;case`MD`:c.searchType=w.METADATA;break;default:c.searchType=w.DEFAULT;break}if(n?c.currentPage=parseInt(n,10):c.currentPage=1,i){let{field:e,direction:t}=this.getSortFieldAndDirection(i);c.selectedSort=_e(e).field,[`asc`,`desc`].includes(t)&&(c.sortDirection=t)}a&&a.forEach(e=>{let[t,...n]=e.split(`:`);if(!n.length)return;let r=n.join(`:`);if(t=t.replace(/Sorter$/,``),t.startsWith(`-`)){o.push(e.slice(1));return}switch(t){case`date`:case`year`:{let[e,n]=r.split(` TO `);e&&n?(c.minSelectedDate=e.substring(1,e.length),c.maxSelectedDate=n.substring(0,n.length-1)):this.setSelectedFacetState(c.selectedFacets,t,r,`selected`);break}case`firstTitle`:c.selectedTitleFilter=r;break;case`firstCreator`:c.selectedCreatorFilter=r;break;default:this.setSelectedFacetState(c.selectedFacets,t,r,`selected`)}}),o&&o.forEach(e=>{let[t,...n]=e.split(`:`);if(!n.length)return;let r=n.join(`:`);this.setSelectedFacetState(c.selectedFacets,t,r,`hidden`)});for(let[t,n]of Object.entries(Ce))if(e.searchParams.get(t)){this.setSelectedFacetState(c.selectedFacets,`clip_type`,n,`selected`);break}return c}getSortFieldAndDirection(e){let t=e.indexOf(` `)>-1,n,r;return t?[n,r]=e.split(` `):(n=e.startsWith(`-`)?e.slice(1):e,r=e.startsWith(`-`)?`desc`:`asc`),{field:n,direction:r}}sortDirectionPrefix(e){return e===`desc`?`-`:``}stripQuotes(e){return e.startsWith(`"`)&&e.endsWith(`"`)?e.substring(1,e.length-1):e}paramsMatch(e,t,n){return n.every(n=>Pe(e.getAll(n).sort(),t.getAll(n).sort()))}removeRecognizedParams(e){e.delete(`query`),e.delete(`sin`),e.delete(`page`),e.delete(`sort`),e.delete(`and[]`),e.delete(`not[]`);for(let t of e.keys())/(and|not)\[\d+\]/.test(t)&&e.delete(t);return e.delete(`q`),e.delete(`search`),e.delete(`only_commercials`),e.delete(`only_factchecks`),e.delete(`only_quotes`),e}hasLegacyParam(e){return e.has(`q`)||e.has(`search`)}setSelectedFacetState(e,t,n,r){if(!Object.prototype.hasOwnProperty.call(e,t))return;let i=e[t];if(!i)return;let a=this.stripQuotes(n);a!==Ie&&(Object.prototype.hasOwnProperty.call(i,a)||(i[a]=this.getDefaultBucket(a)),i[a].state=r)}getDefaultBucket(e){return{key:e,count:0,state:`none`}}},Re=[`forum_posts`,`lending`,`web_archives`],ze=new TextEncoder;async function Be(e){let t=await crypto.subtle.digest(`SHA-1`,ze.encode(e));return[...new Uint8Array(t)].map(e=>e.toString(16).padStart(2,`0`)).join(``)}function Ve(e,t){if(e)for(let[n,r]of Object.entries(e))for(let[i,a]of Object.entries(r))t(n,i,a,e)}function He(e,t,n,r=!1){let i=e??O(),a={...i,[t]:{...i[t],[n.key]:n}};return r&&n.state===`none`&&delete a[t]?.[n.key],a}function Ue(e){let t=O();return Ve(e,(e,n,r)=>{t[e]||(t[e]={}),t[e][n]=r}),t}function We(e,t){let n=Ue(e);return Ve(t,(e,t,r)=>{n[e]||(n[e]={}),n[e][t]=r}),Ve(n,(e,t,r)=>{r.state===`none`&&delete n[e]?.[t]}),n}var Ge=[`selected`,`hidden`,`none`];function Ke(e,t=T.COUNT){return e.sort((e,n)=>{let r=Ge.indexOf(e.state)-Ge.indexOf(n.state),i;return i=t===T.ALPHABETICAL?e.key.localeCompare(n.key):t===T.NUMERIC?Number(n.key)-Number(e.key):n.count-e.count,r||i})}var qe=class{get initialSearchComplete(){return this._initialSearchCompletePromise}constructor(e,t=50){this.host=e,this.pageSize=t,this.pages={},this.offset=0,this.numTileModels=0,this.numInitialPages=2,this.fetchesInProgress=new Set,this.previousQueryKey=``,this.searchResultsLoading=!1,this.facetsLoading=!1,this.facetsReadyToLoad=!1,this.suppressFetches=!1,this.totalResults=0,this.endOfDataReached=!1,this.queryInitialized=!1,this.collectionTitles=new Map,this.tvChannelMaps={},this.tvChannelAliases=new Map,this.parentCollections=[],this.prefixFilterCountMap={},this._initialSearchCompletePromise=Promise.resolve(!0),this.checkAllTiles=()=>{this.map(e=>{let t=e.clone();return t.checked=!0,t})},this.uncheckAllTiles=()=>{this.map(e=>{let t=e.clone();return t.checked=!1,t})},this.removeCheckedTiles=()=>{let{checkedTileModels:e,uncheckedTileModels:t}=this,n=e.length;if(n===0)return;this.offset+=n;let r={},i=Math.floor(this.offset/this.pageSize)+1,a=this.offset%this.pageSize;for(let e=1;e<=i;e+=1){let t=this.offset-this.pageSize*(e-1),n=Math.min(this.pageSize,t);r[e]=Array(n).fill(void 0)}for(let e of t)r[i]||(r[i]=[]),r[i].push(e),a+=1,a>=this.pageSize&&(i+=1,a=0);this.pages=r,this.numTileModels-=n,this.totalResults-=n,this.host.setTileCount(this.size),this.host.setTotalResultCount(this.totalResults),this.requestHostUpdate(),this.refreshVisibleResults()}}hostConnected(){this.setSearchResultsLoading(this.searchResultsLoading),this.setFacetsLoading(this.facetsLoading)}hostUpdate(){if(!this.activeOnHost||(this.setSearchResultsLoading(this.searchResultsLoading),this.setFacetsLoading(this.facetsLoading),!this.host.searchService)||this.pageFetchQueryKey===this.previousQueryKey)return;let e=!this.host.baseQuery;(this.canPerformSearch||e)&&(this.activeOnHost&&this.host.emitQueryStateChanged(),this.handleQueryChange())}get activeOnHost(){return this.host.dataSource===this}get size(){return this.numTileModels}reset(){this.pages={},this.aggregations={},this.histogramAggregation=void 0,this.pageElements=void 0,this.parentCollections=[],this.previousQueryKey=``,this.queryErrorMessage=void 0,this.offset=0,this.numTileModels=0,this.endOfDataReached=!1,this.queryInitialized=!1,this.facetsLoading=!1,this.fetchesInProgress.clear(),this.setTotalResultCount(0),this.requestHostUpdate()}resetPages(){Object.keys(this.pages).length<this.host.maxPagesToManage&&(this.pages={},this.fetchesInProgress.forEach(e=>{e.startsWith(`facets-`)||this.fetchesInProgress.delete(e)}),this.requestHostUpdate())}addPage(e,t){this.pages[e]=t,this.numTileModels+=t.length,this.requestHostUpdate()}addMultiplePages(e,t){let n=Math.ceil(t.length/this.pageSize);for(let r=0;r<n;r+=1){let n=this.pageSize*r;this.addPage(e+r,t.slice(n,n+this.pageSize))}this.host.currentVisiblePageNumbers.some(t=>t>=e&&t<=e+n)&&this.refreshVisibleResults()}getPage(e){return this.pages[e]}getAllPages(){return this.pages}hasPage(e){return!!this.pages[e]}getTileModelAt(e){let t=e+this.offset,n=Math.floor(t/this.pageSize)+1,r=t%this.pageSize,i=1,a=0;for(;a<=t;){if(!this.pages[i])return this.pages[n]?.[r];if(a+this.pages[i].length>t)return this.pages[i][t-a];a+=this.pages[i].length,i+=1}return this.pages[n]?.[r]}indexOf(e){return Object.values(this.pages).flat().indexOf(e)-this.offset}getPageSize(){return this.pageSize}setPageSize(e){this.reset(),this.pageSize=e}setNumInitialPages(e){this.numInitialPages=e}setTotalResultCount(e){this.totalResults=e,this.activeOnHost&&this.host.setTotalResultCount(e)}setFetchesSuppressed(e){this.suppressFetches=e}setEndOfDataReached(e){this.endOfDataReached=e}async handleQueryChange(){if(this.suppressFetches)return;this.reset();let e;this._initialSearchCompletePromise=new Promise(t=>{e=t}),this.queryInitialized=!0,await Promise.all([this.doInitialPageFetch(),this.canFetchFacets?this.fetchFacets():null]),e(!0)}async handleFacetReadinessChange(e){let t=!this.facetsReadyToLoad&&e;this.facetsReadyToLoad=e,t&&this.canFetchFacets&&this.fetchFacets()}get canFetchFacets(){if(this.host.facetLoadStrategy===`off`||Re.includes(this.host.profileElement)||this.host.facetLoadStrategy!==`eager`&&!this.facetsReadyToLoad)return!1;let e=Object.keys(this.aggregations??{}).length>0;return!(this.facetsLoading||e)}map(e){Object.keys(this.pages).length&&(this.pages=Object.fromEntries(Object.entries(this.pages).map(([t,n])=>[t,n.map((t,n,r)=>t&&e(t,n,r))])),this.requestHostUpdate(),this.refreshVisibleResults())}get checkedTileModels(){return this.getFilteredTileModels(e=>e.checked)}get uncheckedTileModels(){return this.getFilteredTileModels(e=>!e.checked)}getFilteredTileModels(e){return Object.values(this.pages).flat().filter((t,n,r)=>t?e(t,n,r):!1)}get effectiveSearchType(){return!this.host.baseQuery?.trim()&&this.host.withinCollection&&this.host.searchType===w.FULLTEXT?w.METADATA:this.host.searchType}get canPerformSearch(){if(!this.host.searchService)return!1;let e=!!this.host.baseQuery?.trim(),t=!!this.host.identifiers?.length,n=!!this.host.withinCollection,r=!!this.host.withinProfile,i=!!this.host.profileElement,a=this.effectiveSearchType,o=a===w.DEFAULT,s=a===w.METADATA,c=a===w.TV;return e||t||n&&(o||s||c)||r&&i&&(o||s)}setSearchResultsLoading(e){this.searchResultsLoading=e,this.activeOnHost&&this.host.setSearchResultsLoading(e)}setFacetsLoading(e){this.facetsLoading=e,this.activeOnHost&&this.host.setFacetsLoading(e)}requestHostUpdate(){this.activeOnHost&&this.host.requestUpdate()}refreshVisibleResults(){this.activeOnHost&&this.host.refreshVisibleResults()}get pageFetchQueryKey(){let e=`pf;${this.host.withinProfile}--pe;${this.host.profileElement}`,t=this.host.withinCollection??e,n=this.host.selectedSort??`none`,r=this.host.sortDirection??`none`;return`fq:${this.fullQuery}-pt:${t}-st:${this.effectiveSearchType}-sf:${n}-sd:${r}`}get facetFetchQueryKey(){let e=`pf;${this.host.withinProfile}--pe;${this.host.profileElement}`,t=this.host.withinCollection??e;return`facets-fq:${this.fullQuery}-pt:${t}-st:${this.effectiveSearchType}`}get filterMap(){let e=new se,{minSelectedDate:t,maxSelectedDate:n,selectedFacets:r,internalFilters:i,selectedTitleFilter:a,selectedCreatorFilter:o}=this.host,s=Fe(this.host.minSelectedDate,this.host.maxSelectedDate);t&&e.addFilter(s,t,ce.GREATER_OR_EQUAL),n&&e.addFilter(s,n,ce.LESS_OR_EQUAL);let c=We(i,r);if(c)for(let[t,n]of Object.entries(c)){let{name:r,values:i}=this.prepareFacetForFetch(t,n);for(let[t,n]of Object.entries(i)){let i;n.state===`selected`?i=ce.INCLUDE:n.state===`hidden`&&(i=ce.EXCLUDE),i&&e.addFilter(r,t,i)}}return a&&e.addFilter(`firstTitle`,a,ce.INCLUDE),o&&e.addFilter(`firstCreator`,o,ce.INCLUDE),e.build()}async requestUID(e,t){return`R:${(await Be(JSON.stringify({pageType:e.pageType,pageTarget:e.pageTarget,query:e.query,fields:e.fields,filters:e.filters,sort:e.sort,searchType:this.host.searchType}))).slice(0,20)}-S:${(await this.host.getSessionId()).slice(0,20)}-P:${e.page??0}-K:${t.charAt(0)}-T:${Date.now()}`}get pageSpecifierParams(){return this.host.identifiers?.length?{pageType:`client_document_fetch`}:this.host.withinCollection?{pageType:`collection_details`,pageTarget:this.host.withinCollection}:this.host.withinProfile?{pageType:`account_details`,pageTarget:this.host.withinProfile,pageElements:this.host.profileElement?[this.host.profileElement]:[]}:null}get fullQuery(){let e=[],t=this.host.baseQuery?.trim();t&&e.push(t),this.host.identifiers&&e.push(`identifier:(${this.host.identifiers.join(` OR `)})`);let{facetQuery:n,dateRangeQueryClause:r,sortFilterQueries:i}=this;return n&&e.push(n),r&&e.push(r),i&&e.push(i),e.join(` AND `).trim()}get facetQuery(){if(!this.host.selectedFacets)return;let e=[];for(let[t,n]of Object.entries(this.host.selectedFacets))e.push(this.buildFacetClause(t,n));return this.joinFacetClauses(e)?.trim()}get dateRangeQueryClause(){if(!(!this.host.minSelectedDate||!this.host.maxSelectedDate))return`${Fe(this.host.minSelectedDate,this.host.maxSelectedDate)}:[${this.host.minSelectedDate} TO ${this.host.maxSelectedDate}]`}get sortFilterQueries(){return[this.titleQuery,this.creatorQuery].filter(e=>e).join(` AND `)}get titleQuery(){return this.host.selectedTitleFilter?`firstTitle:${this.host.selectedTitleFilter}`:void 0}get creatorQuery(){return this.host.selectedCreatorFilter?`firstCreator:${this.host.selectedCreatorFilter}`:void 0}buildFacetClause(e,t){let{name:n,values:r}=this.prepareFacetForFetch(e,t),i=Object.entries(r);if(i.length===0)return``;let a=[];for(let[e,t]of i){let n=t.state===`hidden`?`-`:``;a.push(`${n}"${e}"`)}return`${n}:(${a.join(` OR `)})`}prepareFacetForFetch(e,t){let[n,r]=[e,t];return e===`lending`&&(n=`lending___status`),{name:n,values:r}}joinFacetClauses(e){let t=e.filter(e=>e.length>0);return t.length>0?`(${t.join(` AND `)})`:void 0}async fetchFacets(){let e=this.host.baseQuery?.trim();if(!this.canPerformSearch)return;let{facetFetchQueryKey:t}=this;if(this.fetchesInProgress.has(t))return;this.fetchesInProgress.add(t),this.setFacetsLoading(!0);let n=this.host.sortParam?[this.host.sortParam]:[],r={...this.pageSpecifierParams,query:e||``,identifiers:this.host.identifiers,rows:0,filters:this.filterMap,aggregationsSize:10};r.uid=await this.requestUID({...r,sort:n},`aggregations`);let i=await this.host.searchService?.search(r,this.effectiveSearchType),a=i?.success,o=!this.fetchesInProgress.has(t);if(this.fetchesInProgress.delete(t),o)return;if(!a){let e=i?.error?.message,t=i?.error?.details?.message;!e&&!t&&window?.Sentry?.captureMessage?.(`Missing or malformed facet response from backend`,`error`),this.setFacetsLoading(!1);return}let{aggregations:s,collectionTitles:c,tvChannelAliases:l}=a.response;this.aggregations=s;let u=this.host.searchType===w.TV||this.host.isTVCollection;if(this.histogramAggregation=u?s?.date_histogram??s?.year_histogram:s?.year_histogram,c)for(let[e,t]of Object.entries(c))this.collectionTitles.set(e,t);if(l)for(let[e,t]of Object.entries(l))this.tvChannelAliases.set(e,t);this.setFacetsLoading(!1),this.requestHostUpdate()}async doInitialPageFetch(){this.setSearchResultsLoading(!0),await this.fetchPage(this.host.initialPageNumber,this.numInitialPages)}async fetchPage(e,t=1){let n=this.host.baseQuery?.trim();if(!this.canPerformSearch){this.setSearchResultsLoading(!1);return}if(this.hasPage(e)||this.endOfDataReached)return;let r=e===1?t:1,i=this.pageSize*r,{pageFetchQueryKey:a}=this,o=`${a}-p:${e}`;if(this.fetchesInProgress.has(o))return;for(let t=0;t<r;t+=1)this.fetchesInProgress.add(`${a}-p:${e+t}`);this.previousQueryKey=a;let{withinCollection:s,withinProfile:c}=this.host,l=this.host.sortParam?[this.host.sortParam]:[],u=this.host.selectedSort===E.default,d=this.host.searchType===w.TV&&u;if((c&&u||d)&&this.host.defaultSortField){let{searchServiceKey:e,handledBySearchService:t}=D[this.host.defaultSortField];e&&t&&(l=[{field:e,direction:this.host.defaultSortDirection??`desc`}])}let f={...this.pageSpecifierParams,query:n||``,identifiers:this.host.identifiers,page:e,rows:i,sort:l,filters:this.filterMap,aggregations:{omit:!0}};f.uid=await this.requestUID(f,`hits`);let p=await this.host.searchService?.search(f,this.effectiveSearchType),m=p?.success;if(!this.fetchesInProgress.has(o))return;for(let t=0;t<r;t+=1)this.fetchesInProgress.delete(`${a}-p:${e+t}`);if(!m){let e=p?.error?.message,t=p?.error?.details?.message;this.queryErrorMessage=`${e??``}${t?`; ${t}`:``}`,this.queryErrorMessage||(this.queryErrorMessage=`Missing or malformed response from backend`,window?.Sentry?.captureMessage?.(this.queryErrorMessage,`error`)),this.setSearchResultsLoading(!1),this.requestHostUpdate(),this.host.emitSearchError();return}this.setTotalResultCount(m.response.totalResults-this.offset),this.activeOnHost&&this.totalResults===0&&this.host.emitEmptyResults(),this.sessionContext=m.sessionContext,s?(this.collectionExtraInfo=m.response.collectionExtraInfo,this.activeOnHost&&this.host.applyDefaultCollectionSort(this.collectionExtraInfo),this.collectionExtraInfo&&(this.parentCollections=[].concat(this.collectionExtraInfo.public_metadata?.collection??[]),this.host.isTVCollection=this.host.withinCollection?.startsWith(`TV-`)||this.host.withinCollection===`tvnews`||this.host.withinCollection===`tvarchive`||this.parentCollections.includes(`tvnews`)||this.parentCollections.includes(`tvarchive`))):c&&(this.accountExtraInfo=m.response.accountExtraInfo,this.pageElements=m.response.pageElements);let{results:h,collectionTitles:g,tvChannelAliases:_}=m.response;if(h&&h.length>0){if(g){for(let[e,t]of Object.entries(g))this.collectionTitles.set(e,t);let e=this.collectionExtraInfo?.public_metadata?.title;s&&e&&this.collectionTitles.set(s,e)}if(_)for(let[e,t]of Object.entries(_))this.tvChannelAliases.set(e,t);let t=[`lending`,`web_archives`].includes(this.host.profileElement);t&&(r=Math.ceil(h.length/this.pageSize),this.endOfDataReached=!0,this.activeOnHost&&this.host.setTileCount(this.totalResults));for(let n=0;n<r;n+=1){let i=this.pageSize*n;this.addFetchedResultsToDataSource(e+n,h.slice(i,i+this.pageSize),!t||n===r-1)}}(this.size>=this.totalResults||h.length===0)&&(this.endOfDataReached=!0,this.activeOnHost&&this.host.setTileCount(this.size)),this.setSearchResultsLoading(!1),this.requestHostUpdate()}get hitRequestSource(){let{host:e}=this;return e.baseQuery?`search_query`:e.withinProfile?`profile_tab`:e.withinCollection?`collection_members`:`unknown`}addFetchedResultsToDataSource(e,t,n=!0){let r=[],i=this.hitRequestSource;t?.forEach(e=>{e.identifier&&r.push(new ge(e,i))}),this.addPage(e,r),n&&this.refreshVisibleResults()}async fetchPrefixFilterBuckets(e){let t=this.host.baseQuery?.trim();if(!this.canPerformSearch)return[];let n=Se[e],r=this.host.sortParam?[this.host.sortParam]:[],i={...this.pageSpecifierParams,query:t||``,identifiers:this.host.identifiers,rows:0,filters:this.filterMap,aggregations:{simpleParams:[n]},aggregationsSize:26};return i.uid=await this.requestUID({...i,sort:r},`aggregations`),(await this.host.searchService?.search(i,this.effectiveSearchType))?.success?.response?.aggregations?.[n]?.buckets??[]}async updatePrefixFilterCounts(e){let{facetFetchQueryKey:t}=this,n=await this.fetchPrefixFilterBuckets(e);t===this.facetFetchQueryKey&&(this.prefixFilterCountMap={...this.prefixFilterCountMap},this.prefixFilterCountMap[e]=n.reduce((e,t)=>(e[t.key.toUpperCase()]=t.doc_count,e),{}),this.requestHostUpdate())}async updatePrefixFiltersForCurrentSort(){if([`title`,`creator`].includes(this.host.selectedSort)){let e=this.host.selectedSort;this.prefixFilterCountMap[e]||this.updatePrefixFilterCounts(e)}}refreshLetterCounts(){Object.keys(this.prefixFilterCountMap).length>0&&(this.prefixFilterCountMap={}),this.updatePrefixFiltersForCurrentSort(),this.requestHostUpdate()}populateTVChannelMaps(){return this._tvMapsPromise||(this._tvMapsPromise=this._fetchTVChannelMaps()),this._tvMapsPromise}async _fetchTVChannelMaps(){let e=`https://av.archive.org/etc`,t=new Date().toISOString().slice(0,10),n=fetch(`${e}/chan2network.json?date=${t}`),r=fetch(`${e}/program2chans.json?date=${t}`),[i,a]=await Promise.all([n,r]);return this.tvChannelMaps.channelToNetwork=new Map(Object.entries(await i.json())),this.tvChannelMaps.programToChannels=new Map(Object.entries(await a.json())),this.requestHostUpdate(),this.tvChannelMaps}},k;(function(e){e.default=`collection-browser`})(k||(k={}));var A;(function(e){e.sortBy=`sortBy`,e.filterByCreator=`filterByCreator`,e.filterByTitle=`filterByTitle`,e.displayMode=`displayMode`,e.loadDesktopView=`loadDesktopView`,e.loadMobileView=`loadMobileView`,e.facetSelected=`facetSelected`,e.facetDeselected=`facetDeselected`,e.facetNegativeSelected=`facetNegativeSelected`,e.facetNegativeDeselected=`facetNegativeDeselected`,e.mobileFacetsToggled=`mobileFacetsToggled`,e.partOfCollectionClicked=`partOfCollectionClicked`,e.histogramChanged=`histogramChanged`,e.histogramChangedFromModal=`histogramChangedFromModal`,e.histogramExpanded=`histogramExpanded`,e.resultSelected=`resultSelected`,e.moreFacetsPageChange=`moreFacetsPageChange`,e.showMoreFacetsModal=`showMoreFacetsModal`,e.closeMoreFacetsModal=`closeMoreFacetsModal`,e.applyMoreFacetsModal=`applyMoreFacetsModal`})(A||(A={}));var Je=m`<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><path d="m79.8883285 50.0035012.1116715-.1085359-43.1159942-46.61088155c-2.401537-2.18938917-4.6902018-3.28408375-6.8659943-3.28408375s-4.1642651.63837733-5.9654178 1.91513199c-1.8011528 1.27675467-3.1520173 2.97248092-4.0525937 5.08717877l39.4020173 42.99768924-39.4020173 42.9976892c.9005764 2.1146979 2.2514409 3.8104241 4.0525937 5.0871788 1.8011527 1.2767547 3.7896253 1.915132 5.9654178 1.915132 2.1013449 0 4.3900096-1.0573489 6.8659943-3.1720468l43.1159942-46.7194174z"/></svg>
`,j=e`
  .sr-only {
    position: absolute !important;
    width: 1px !important;
    height: 1px !important;
    margin: -1px !important;
    padding: 0 !important;
    border: 0 !important;
    overflow: hidden !important;
    white-space: nowrap !important;
    clip: rect(1px, 1px, 1px, 1px) !important;
    -webkit-clip-path: inset(50%) !important;
    clip-path: inset(50%) !important;
    user-select: none !important;
  }
`;function M(){return o(`en`,{desc:`Not shown to users. Reports the active locale: translators should translate this to the ISO code of the locale they are translating into.`})}function Ye(e,t){return t.some(t=>e.has(t))}function Xe(e,t){let n=[...e],r=[...t],i=n.length,a=r.length;if(i===0)return!0;let o=0,s=0;for(;s<a;){if(r[s]===n[o]&&(o+=1),o>=i)return!0;s+=1}return!1}var Ze=m`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 4" width="1em" height="1em" fill="currentColor" aria-hidden="true" focusable="false"><path
    d="m6.7226499.58397485c.22976435-.15317623.54019902-.09108929.69337525.13867505.13615665.20423498.10222882.47220947-.06836249.63681849l-.07031256.05655676-3.2773501 2.18490006-3.2773501-2.18490006c-.22976434-.15317623-.29185128-.4636109-.13867505-.69337525.13615665-.20423497.39656688-.27598409.61412572-.18182636l.07924953.04315131 2.7226499 1.81402515z"
  ></path></svg>`,Qe=m`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 4" width="1em" height="1em" fill="currentColor" aria-hidden="true" focusable="false"><path
    d="m6.7226499 3.51689722c.22976435.15317623.54019902.0910893.69337525-.13867505.13615665-.20423497.10222882-.47220946-.06836249-.63681849l-.07031256-.05655675-3.2773501-2.18490007-3.2773501 2.18490007c-.22976434.15317623-.29185128.4636109-.13867505.69337524.13615665.20423498.39656688.27598409.61412572.18182636l.07924953-.04315131 2.7226499-1.81402514z"
  ></path></svg>`,$e=m`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="1em" height="1em" fill="currentColor" aria-hidden="true" focusable="false"><path
    d="m50 0c27.6142375 0 50 22.3857625 50 50s-22.3857625 50-50 50-50-22.3857625-50-50 22.3857625-50 50-50zm23.8159475 26.1840525c-1.4033215-1.4033215-3.5816761-1.5592461-5.1572272-.4677738l-.5598841.4677738-18.0988362 18.0989475-18.0988362-18.0989475-.5598841-.4677738c-1.5755511-1.0914723-3.7539057-.9355477-5.1572272.4677738-1.5787367 1.5787367-1.5787367 4.1383746 0 5.7171113l18.0989475 18.0988362-18.0989475 18.0988362c-1.5787367 1.5787367-1.5787367 4.1383746 0 5.7171113 1.4033215 1.4033215 3.5816761 1.5592461 5.1572272.4677738l.5598841-.4677738 18.0988362-18.0989475 18.0988362 18.0989475.5598841.4677738c1.5755511 1.0914723 3.7539057.9355477 5.1572272-.4677738 1.5787367-1.5787367 1.5787367-4.1383746 0-5.7171113l-18.0989475-18.0988362 18.0989475-18.0988362c1.5787367-1.5787367 1.5787367-4.1383746 0-5.7171113z"
    fill-rule="evenodd"
  ></path></svg>`,et={all:()=>!0,prefix:(e,t)=>t.startsWith(e),suffix:(e,t)=>t.endsWith(e),substring:(e,t)=>t.includes(e),subsequence:Xe},tt=`list`,nt=`substring`,rt=e=>e,it=e=>e.toLocaleLowerCase(),N=class extends c{constructor(){super(),this.options=[],this.behavior=tt,this.maxAutocompleteEntries=1/0,this.filter=nt,this.caseSensitive=!1,this.sort=!1,this.wrapArrowKeys=!1,this.stayOpen=!1,this.clearable=!1,this.open=!1,this.disabled=!1,this.required=!1,this.value=null,this.hasFocus=!1,this.highlightedOption=null,this.enteredText=``,this.filterText=``,this.losingFocus=!1,this.optionsByID=new Map,this.optionFilteringValues=new Map,this.optionsRespectingSortFlag=[],this.filteredOptions=[],this.internals=this.attachInternals()}render(){let e=g({disabled:this.disabled,focused:this.hasFocus});return a`
      <div id="container" part="container">
        ${this.labelTemplate}
        <div id="main-widget-row" class=${e} part="combo-box">
          ${this.textInputTemplate}
          ${this.clearable?this.clearButtonTemplate:n}
          ${this.caretButtonTemplate}
        </div>
        ${this.optionsListTemplate}
      </div>
    `}willUpdate(e){(e.has(`options`)||e.has(`caseSensitive`))&&this.rebuildOptionFilteringValues(),e.has(`options`)&&this.rebuildOptionIDMap(),(e.has(`options`)||e.has(`sort`))&&this.rebuildSortedOptions(),Ye(e,[`options`,`behavior`,`maxAutocompleteEntries`,`filter`,`filterText`,`caseSensitive`,`sort`])&&this.rebuildFilteredOptions(),e.has(`open`)&&(this.open?this.value&&this.setHighlightedOption(this.selectedOption):this.setHighlightedOption(null)),e.has(`required`)&&this.updateFormValidity()}updated(e){e.has(`value`)&&this.handleValueChanged(),e.has(`options`)&&this.behavior!==`freeform`&&!this.selectedOption&&this.clearSelectedOption(),e.has(`open`)&&(this.open?(this.positionOptionsMenu(),this.optionsList?.showPopover?.(),this.optionsList?.classList.add(`visible`)):(this.optionsList?.hidePopover?.(),this.optionsList?.classList.remove(`visible`)))}get labelTemplate(){return a`
      <label id="label" for="text-input" part="label">
        <slot name="label"></slot>
      </label>
    `}get textInputTemplate(){return a`
      <input
        type="text"
        id="text-input"
        class=${g({"clear-padding":this.clearable&&!this.shouldShowClearButton})}
        .value=${_(this.enteredText)}
        placeholder=${b(this.placeholder)}
        part="text-input"
        role="combobox"
        autocomplete="off"
        aria-autocomplete="list"
        aria-controls="options-list"
        aria-expanded=${this.open}
        aria-activedescendant=${b(this.highlightedOption?.id)}
        ?readonly=${this.behavior===`select-only`}
        ?disabled=${this.disabled}
        ?required=${this.required}
        @click=${this.handleComboBoxClick}
        @keydown=${this.handleComboBoxKeyDown}
        @input=${this.handleTextBoxInput}
        @focus=${this.handleFocus}
        @blur=${this.handleBlur}
      />
    `}get clearButtonTemplate(){return a`
      <button
        type="button"
        id="clear-button"
        part="clear-button"
        tabindex="-1"
        ?hidden=${!this.shouldShowClearButton}
        @click=${this.handleClearButtonClick}
      >
        <span class="sr-only">${o(`Clear`)}</span>
        <slot name="clear-button">
          <span
            class="icon clear-icon"
            part="icon clear-icon"
            aria-hidden="true"
            >${$e}</span
          >
        </slot>
      </button>
    `}get caretTemplate(){return a`
      <slot name="caret-closed" ?hidden=${this.open}>
        <span class="icon caret-icon" part="icon caret-icon" aria-hidden="true"
          >${Ze}</span
        >
      </slot>
      <slot name="caret-open" ?hidden=${!this.open}>
        <span class="icon caret-icon" part="icon caret-icon" aria-hidden="true"
          >${Qe}</span
        >
      </slot>
    `}get caretButtonTemplate(){return a`
      <button
        type="button"
        id="caret-button"
        part="caret-button"
        tabindex="-1"
        aria-controls="options-list"
        aria-expanded=${this.open}
        ?disabled=${this.disabled}
        @click=${this.handleComboBoxClick}
        @keydown=${this.handleComboBoxKeyDown}
        @focus=${this.handleFocus}
        @blur=${this.handleBlur}
      >
        <span class="sr-only">${o(`Toggle options`)}</span>
        ${this.caretTemplate}
      </button>
    `}get optionsListTemplate(){return a`
      <ul
        id="options-list"
        part="options-list"
        role="listbox"
        tabindex="-1"
        popover
        ?hidden=${!this.open}
        @focus=${this.handleFocus}
        @blur=${this.handleBlur}
      >
        <slot name="options-list-top"></slot>
        ${l(this.open,()=>this.optionTemplates)}
        <slot name="options-list-bottom"></slot>
      </ul>
    `}get optionTemplates(){return this.filteredOptions.length===0&&this.maxAutocompleteEntries>0?[this.emptyOptionsTemplate]:this.filteredOptions.map(e=>{let t=e===this.highlightedOption,n=g({option:!0,highlight:t});return a`
        <li
          id=${e.id}
          class=${n}
          part="option ${t?`highlight`:``}"
          role="option"
          tabindex="-1"
          @pointerenter=${this.handleOptionPointerEnter}
          @pointermove=${this.handleOptionPointerMove}
          @click=${this.handleOptionClick}
          @focus=${this.handleFocus}
          @blur=${this.handleBlur}
        >
          ${e.content??e.text}
        </li>
      `})}get emptyOptionsTemplate(){return a`
      <li id="empty-options" part="empty-options">
        <slot name="empty-options">${o(`No matching options`)}</slot>
      </li>
    `}handleOptionPointerEnter(e){this.handleOptionPointerMove(e)}handleOptionPointerMove(e){let t=e.currentTarget,n=this.getOptionFor(t.id);n&&this.setHighlightedOption(n)}handleOptionClick(e){let t=e.currentTarget,n=this.getOptionFor(t.id);n&&(this.setSelectedOption(n.id),this.stayOpen||this.closeOptionsMenu())}handleComboBoxKeyDown(e){switch(e.key){case`Enter`:this.handleEnterPressed();break;case`Escape`:this.handleEscapePressed();break;case`ArrowUp`:e.altKey?this.handleAltUpArrowPressed():this.handleUpArrowPressed();break;case`ArrowDown`:e.altKey?this.handleAltDownArrowPressed():this.handleDownArrowPressed();break;case`Tab`:this.handleTabPressed();return;case` `:this.handleSpacePressed(e);return;default:return}e.stopPropagation(),e.preventDefault()}async handleTextBoxInput(){let e=this.textInput?.value??``;this.enteredText=e,this.setFilterText(e),this.openOptionsMenu(),await this.updateComplete,this.highlightFirstOption()}handleEnterPressed(){if(!this.open){this.openOptionsMenu();return}this.highlightedOption?this.setSelectedOption(this.highlightedOption.id):this.behavior===`freeform`&&this.setValue(this.enteredText),this.stayOpen||(this.open=!1)}handleEscapePressed(){if(this.open){this.closeOptionsMenu();return}this.clearSelectedOption()}handleUpArrowPressed(){this.open||this.openOptionsMenu(),this.highlightPreviousOption()}handleDownArrowPressed(){this.open||this.openOptionsMenu(),this.highlightNextOption()}handleAltUpArrowPressed(){this.closeOptionsMenu()}handleAltDownArrowPressed(){this.openOptionsMenu()}handleTabPressed(){this.highlightedOption&&(this.setSelectedOption(this.highlightedOption.id),this.stayOpen||(this.open=!1))}handleSpacePressed(e){this.behavior===`select-only`&&(this.open?this.highlightedOption&&(this.setSelectedOption(this.highlightedOption.id),this.stayOpen||(this.open=!1)):this.openOptionsMenu(),e.stopPropagation(),e.preventDefault())}handleComboBoxClick(){this.toggleOptionsMenu()}handleClearButtonClick(){this.clearSelectedOption(),this.textInput?.focus(),this.openOptionsMenu()}handleFocus(){this.behavior!==`select-only`&&this.textInput?.focus(),this.hasFocus=!0,this.losingFocus=!1}handleBlur(){this.hasFocus=!1,this.losingFocus=!0,setTimeout(()=>{this.losingFocus&&!this.shadowRoot?.activeElement&&(this.losingFocus=!1,this.closeOptionsMenu(),this.behavior===`list`?this.setTextValue(this.selectedOption?.text??``,!1):this.behavior===`freeform`&&(this.enteredText||this.value)&&this.setValue(this.enteredText))},0)}handleValueChanged(){if(this.value==null){this.enteredText&&this.setTextValue(``,!1);return}let e=this.getOptionFor(this.value);if(this.behavior===`freeform`){let t=e?.text??this.value;t!==this.enteredText&&this.setTextValue(t);return}if(!e){this.clearSelectedOption();return}this.enteredText!==e.text&&(this.setTextValue(e.text,!1),this.setFilterText(``))}highlightFirstOption(){this.setHighlightedOption(this.firstFilteredOption)}highlightLastOption(){this.setHighlightedOption(this.lastFilteredOption)}highlightPreviousOption(){let{filteredOptions:e,lastFilteredIndex:t}=this;if(!this.highlightedOption){this.highlightLastOption();return}let{highlightedIndex:n}=this,r=this.wrapArrowKeys&&n===0?t:Math.max(n-1,0);this.setHighlightedOption(e[r])}highlightNextOption(){let{filteredOptions:e,lastFilteredIndex:t}=this;if(!this.highlightedOption){this.highlightFirstOption();return}let{highlightedIndex:n}=this,r=this.wrapArrowKeys&&n===t?0:Math.min(n+1,t);this.setHighlightedOption(e[r])}async setHighlightedOption(e){this.highlightedOption=e,await this.updateComplete;let{optionsList:t,highlightedElement:n}=this;if(!n||!t)return;let r=n.getBoundingClientRect(),i=t.getBoundingClientRect();(r.top<i.top||r.bottom>i.bottom)&&n.scrollIntoView({block:`nearest`})}setSelectedOption(e){let t=this.getOptionFor(e);if(!t)throw RangeError(`Unknown option ID`);let n=this.value;this.value=t.id,this.internals.setFormValue(this.value),this.setTextValue(t.text,!1),this.setFilterText(``),this.value!==n&&this.emitChangeEvent(),t.onSelected?.(t)}clearSelectedOption(){let e=this.value;this.value=null,this.internals.setFormValue(this.value),this.setTextValue(``),this.value!==e&&this.emitChangeEvent()}setValue(e){if(this.behavior===`freeform`){let t=this.value;this.value=e,this.internals.setFormValue(this.value),this.setTextValue(e),this.value!==t&&this.emitChangeEvent()}else this.setSelectedOption(e)}setTextValue(e,t=!0){this.textInput&&(this.textInput.value=e),this.enteredText=e,t&&this.setFilterText(e)}setFilterText(e){let{caseTransform:t}=this;this.filterText=t(e)}openOptionsMenu(){this.open=!0,this.emitToggleEvent()}closeOptionsMenu(){this.open=!1,this.emitToggleEvent()}toggleOptionsMenu(){this.open=!this.open,this.emitToggleEvent()}updateFormValidity(){this.required&&!this.value?this.internals.setValidity({valueMissing:!0},o(`A value is required`)):this.internals.setValidity({})}emitChangeEvent(){this.dispatchEvent(new CustomEvent(`change`,{detail:this.value}))}emitToggleEvent(){this.dispatchEvent(new CustomEvent(`toggle`,{detail:this.open}))}get isEmpty(){return!this.selectedOption&&!this.enteredText}get shouldShowClearButton(){return this.clearable&&!this.disabled&&!this.isEmpty}positionOptionsMenu(){let{mainWidgetRow:e,optionsList:t}=this;if(!e||!t)return;let n=e.getBoundingClientRect(),{innerHeight:r,scrollX:i,scrollY:a}=window,o=n.top,s=r-n.bottom,c=`var(--combo-box-list-max-height--)`,l={top:`${n.bottom+a}px`,left:`${n.left+i}px`,width:`var(--combo-box-list-width--, ${n.width}px)`,maxHeight:`min(${c}, ${s}px)`};Object.assign(t.style,l),setTimeout(()=>{t.getBoundingClientRect().bottom>=r&&o>s&&(t.style.top=`auto`,t.style.bottom=`${r-n.top-a}px`,t.style.maxHeight=`min(${c}, ${o}px)`)},0)}get caseTransform(){return this.caseSensitive?rt:it}getOptionFor(e){return this.optionsByID.get(e)??null}rebuildOptionIDMap(){this.optionsByID.clear();for(let e of this.options)this.optionsByID.set(e.id,e)}rebuildSortedOptions(){this.sort?this.optionsRespectingSortFlag=[...this.options].sort((e,t)=>{let n=this.optionFilteringValues.get(e),r=this.optionFilteringValues.get(t);return n.localeCompare(r)}):this.optionsRespectingSortFlag=this.options}rebuildOptionFilteringValues(){this.optionFilteringValues.clear();let{caseTransform:e}=this;for(let t of this.options){let n=e(t.text);this.optionFilteringValues.set(t,n)}}rebuildFilteredOptions(){let e=this.behavior===`select-only`?`all`:this.filter,t=typeof e==`string`?et[e]:e,n=this.optionsRespectingSortFlag.filter(e=>{let n=this.optionFilteringValues.get(e);return n?t(this.filterText,n,e):!1}).slice(0,this.maxAutocompleteEntries);this.filteredOptions=n}get firstFilteredOption(){return this.filteredOptions[0]??null}get lastFilteredOption(){return this.filteredOptions[this.lastFilteredIndex]??null}get lastFilteredIndex(){return this.filteredOptions.length-1}get selectedOption(){return this.value==null?null:this.getOptionFor(this.value)}get highlightedIndex(){return this.highlightedOption?this.filteredOptions.indexOf(this.highlightedOption):-1}get highlightedElement(){return this.highlightedOption?this.shadowRoot.getElementById(this.highlightedOption.id):null}static get styles(){return[ae,e`
      :host {
        --combo-box-width--: var(--combo-box-width);
        --combo-box-padding--: var(--padding-sm);
        --combo-box-list-width--: var(--combo-box-list-width, unset);
        --combo-box-list-max-height--: var(--combo-box-list-max-height, 250px);
        --combo-box-list-fade-duration--: var(
          --combo-box-list-fade-duration,
          125ms
        );
      }

      #container {
        display: inline-block;
        width: var(--combo-box-width--);
      }

      #label {
        display: block;
        width: fit-content;
      }

      #main-widget-row {
        display: inline-flex;
        align-items: stretch;
        flex-wrap: nowrap;
        background: white;
        border: 1px solid black;
        width: 100%;
      }

      #main-widget-row:not(.focused, .disabled):hover,
      #main-widget-row:not(.focused, .disabled):active {
        background: #fafafa;
      }

      #main-widget-row.focused {
        outline: black auto 1px;
        outline-offset: 3px;
      }

      #main-widget-row.disabled {
        background: #f4f4f4;
        border-color: #a0a0a0;
        color: #404040;
      }

      #text-input {
        appearance: none;
        background: transparent;
        border: none;
        padding: var(--combo-box-padding--);
        padding-right: 0;
        width: 100%;
        font-size: inherit;
        font-family: inherit;
        font-weight: inherit;
        font-style: inherit;
        color: inherit;
        outline: none;
        text-overflow: ellipsis;
      }

      #text-input.clear-padding {
        padding-right: 30px;
      }

      #text-input:read-only {
        cursor: pointer;
      }

      #clear-button,
      #caret-button {
        display: inline-flex;
        align-items: center;
        appearance: none;
        background: transparent;
        border: none;
        padding: var(--combo-box-padding--) 5px;
        outline: none;
        font-size: inherit;
        font-family: inherit;
        font-weight: inherit;
        font-style: inherit;
        cursor: pointer;
      }

      #clear-button {
        flex: 0 0 30px;
      }

      #clear-button[hidden] {
        display: none;
      }

      #caret-button {
        padding-right: var(--combo-box-padding--);
      }

      #options-list {
        position: absolute;
        list-style-type: none;
        margin: 1px 0 0;
        border: none;
        padding: 0;
        background: white;
        width: var(--combo-box-list-width--);
        max-height: 400px;
        box-shadow: 0 0 1px 1px #ddd;
        opacity: 0;
        transition: opacity var(--combo-box-list-fade-duration--) ease;
      }

      #options-list.visible {
        opacity: 1;
      }

      #empty-options {
        padding: 5px;
        color: #606060;
        font-style: italic;
        text-align: center;
      }

      /* The svg fills this box, so its width and height are the glyph's size. */
      .icon {
        display: inline-block;
        color: #000;
      }

      .icon > svg {
        display: block;
        width: 100%;
        height: 100%;
      }

      .caret-icon {
        width: 0.875em;
        height: 0.875em;
      }

      .clear-icon {
        width: 1em;
        height: 1em;
      }

      .option {
        padding: 7px 5px;
        width: 100%;
        box-sizing: border-box;
        line-height: 1.1;
        text-overflow: ellipsis;
        overflow: hidden;
        cursor: pointer;
      }

      .highlight {
        background-color: #dbe0ff;
      }

      .disabled,
      .disabled * {
        cursor: not-allowed !important;
      }

      .sr-only {
        position: absolute !important;
        width: 1px !important;
        height: 1px !important;
        margin: -1px !important;
        padding: 0 !important;
        border: 0 !important;
        overflow: hidden !important;
        white-space: nowrap !important;
        clip: rect(1px, 1px, 1px, 1px) !important;
        -webkit-clip-path: inset(50%) !important;
        clip-path: inset(50%) !important;
        user-select: none !important;
      }
    `]}};N.formAssociated=!0,N.shadowRootOptions={...c.shadowRootOptions,delegatesFocus:!0},C([t({type:Array})],N.prototype,`options`,void 0),C([t({type:String})],N.prototype,`placeholder`,void 0),C([t({type:String})],N.prototype,`behavior`,void 0),C([t({type:Number,attribute:`max-autocomplete-entries`})],N.prototype,`maxAutocompleteEntries`,void 0),C([t({type:String})],N.prototype,`filter`,void 0),C([t({type:Boolean,reflect:!0,attribute:`case-sensitive`})],N.prototype,`caseSensitive`,void 0),C([t({type:Boolean,reflect:!0})],N.prototype,`sort`,void 0),C([t({type:Boolean,reflect:!0,attribute:`wrap-arrow-keys`})],N.prototype,`wrapArrowKeys`,void 0),C([t({type:Boolean,reflect:!0,attribute:`stay-open`})],N.prototype,`stayOpen`,void 0),C([t({type:Boolean,reflect:!0})],N.prototype,`clearable`,void 0),C([t({type:Boolean,reflect:!0})],N.prototype,`open`,void 0),C([t({type:Boolean,reflect:!0})],N.prototype,`disabled`,void 0),C([t({type:Boolean,reflect:!0})],N.prototype,`required`,void 0),C([t({type:String})],N.prototype,`value`,void 0),C([s()],N.prototype,`hasFocus`,void 0),C([s()],N.prototype,`highlightedOption`,void 0),C([s()],N.prototype,`enteredText`,void 0),C([s()],N.prototype,`filterText`,void 0),C([S(`#main-widget-row`)],N.prototype,`mainWidgetRow`,void 0),C([S(`#text-input`)],N.prototype,`textInput`,void 0),C([S(`#options-list`)],N.prototype,`optionsList`,void 0),N=C([ie(`ia-combo-box`),i()],N);var at=m`
  <svg viewBox="0 0 787 400" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><defs><path id="a" d="m0 127.511499c0-70.3329329 57.1960466-127.511499 127.51918-127.511499 70.246413 0 127.48082 57.1785661 127.48082 127.511499 0 70.294604-57.234407 127.488501-127.48082 127.488501-70.3231334 0-127.51918-57.193897-127.51918-127.488501z"/><mask id="b" fill="#fff"><use fill="#fff" fill-rule="evenodd" xlink:href="#a"/></mask></defs><g fill="none" fill-rule="evenodd"><g transform="translate(0 79)"><path d="m180 161h13v18h-13z" fill="#ffcd27" opacity=".6"/><path d="m162 161h13v18h-13z" fill="#ffcd27" opacity=".5"/><path d="m144 161h13v18h-13z" fill="#ffcd27" opacity=".5"/><path d="m126 161h13v18h-13z" fill="#ffcd27" opacity=".4"/><path d="m108 161h13v18h-13z" fill="#ffcd27" opacity=".4"/><path d="m90 161h13v18h-13z" fill="#f1644b" opacity=".3"/><path d="m72 161h13v18h-13z" fill="#ffcd27" opacity=".3"/><path d="m54 161h13v18h-13z" fill="#ffcd27" opacity=".2"/><path d="m36 161h13v18h-13z" fill="#ffcd27" opacity=".2"/><path d="m18 161h13v18h-13z" fill="#ffcd27" opacity=".1"/><path d="m0 161h13v18h-13z" fill="#f1644b" opacity=".1"/><path d="m180 138h13v18h-13z" fill="#faab3c" opacity=".6"/><path d="m162 138h13v18h-13z" fill="#faab3c" opacity=".5"/><path d="m144 138h13v18h-13z" fill="#faab3c" opacity=".5"/><path d="m126 138h13v18h-13z" fill="#aa99c9" opacity=".4"/><path d="m108 138h13v18h-13z" fill="#faab3c" opacity=".4"/><path d="m90 138h13v18h-13z" fill="#faab3c" opacity=".3"/><g fill="#f1644b"><path d="m72 138h13v18h-13z" opacity=".3"/><path d="m54 138h13v18h-13z" opacity=".2"/><path d="m36 138h13v18h-13z" opacity=".2"/><path d="m18 138h13v18h-13z" opacity=".1"/><path d="m0 138h13v18h-13z" opacity=".1"/><path d="m180 115h13v18h-13z" opacity=".6"/><path d="m162 115h13v18h-13z" opacity=".5"/><path d="m144 115h13v18h-13z" opacity=".5"/><path d="m126 115h13v18h-13z" opacity=".4"/><path d="m108 115h13v18h-13z" opacity=".4"/><path d="m90 115h13v18h-13z" opacity=".3"/><path d="m72 115h13v18h-13z" opacity=".3"/></g><path d="m54 115h13v18h-13z" fill="#9ecc4f" opacity=".2"/><path d="m36 115h13v18h-13z" fill="#f1644b" opacity=".2"/><path d="m18 115h13v18h-13z" fill="#f1644b" opacity=".1"/><path d="m0 115h13v18h-13z" fill="#f1644b" opacity=".1"/><path d="m180 92h13v18h-13z" fill="#333" opacity=".6"/><path d="m162 92h13v18h-13z" fill="#333" opacity=".5"/><path d="m144 92h13v18h-13z" fill="#9ecc4f" opacity=".5"/><path d="m126 92h13v18h-13z" fill="#aa99c9" opacity=".4"/><path d="m108 92h13v18h-13z" fill="#f1644b" opacity=".4"/><path d="m90 92h13v18h-13z" fill="#faab3c" opacity=".3"/><path d="m72 92h13v18h-13z" fill="#faab3c" opacity=".3"/><path d="m54 92h13v18h-13z" fill="#faab3c" opacity=".2"/><path d="m36 92h13v18h-13z" fill="#faab3c" opacity=".2"/><path d="m18 92h13v18h-13z" fill="#f1644b" opacity=".1"/><path d="m0 92h13v18h-13z" fill="#ffcd27" opacity=".1"/><path d="m180 69h13v18h-13z" fill="#f1644b" opacity=".6"/><path d="m162 69h13v18h-13z" fill="#333" opacity=".5"/><path d="m144 69h13v18h-13z" fill="#9ecc4f" opacity=".5"/><path d="m126 69h13v18h-13z" fill="#333" opacity=".4"/><path d="m108 69h13v18h-13z" fill="#333" opacity=".4"/><path d="m90 69h13v18h-13z" fill="#00adef" opacity=".3"/><path d="m72 69h13v18h-13z" fill="#00adef" opacity=".3"/><path d="m54 69h13v18h-13z" fill="#00adef" opacity=".2"/><path d="m36 69h13v18h-13z" fill="#333" opacity=".2"/><path d="m18 69h13v18h-13z" fill="#9ecc4f" opacity=".1"/><path d="m0 69h13v18h-13z" fill="#ffcd27" opacity=".1"/><path d="m180 46h13v18h-13z" fill="#00adef" opacity=".6"/><path d="m162 46h13v18h-13z" fill="#00adef" opacity=".5"/><path d="m144 46h13v18h-13z" fill="#9ecc4f" opacity=".5"/><path d="m126 46h13v18h-13z" fill="#333" opacity=".4"/><path d="m108 46h13v18h-13z" fill="#333" opacity=".4"/><path d="m90 46h13v18h-13z" fill="#ffcd27" opacity=".3"/><path d="m72 46h13v18h-13z" fill="#333" opacity=".3"/><path d="m54 46h13v18h-13z" fill="#aa99c9" opacity=".2"/><path d="m36 46h13v18h-13z" fill="#faab3c" opacity=".2"/><path d="m18 46h13v18h-13z" fill="#faab3c" opacity=".1"/><path d="m0 46h13v18h-13z" fill="#333" opacity=".1"/><path d="m180 23h13v18h-13z" fill="#00adef" opacity=".6"/><path d="m162 23h13v18h-13z" fill="#00adef" opacity=".5"/><path d="m144 23h13v18h-13z" fill="#9ecc4f" opacity=".5"/><path d="m126 23h13v18h-13z" fill="#f1644b" opacity=".4"/><path d="m108 23h13v18h-13z" fill="#faab3c" opacity=".4"/><path d="m90 23h13v18h-13z" fill="#faab3c" opacity=".3"/><path d="m72 23h13v18h-13z" fill="#f1644b" opacity=".3"/><path d="m54 23h13v18h-13z" fill="#f1644b" opacity=".2"/><path d="m36 23h13v18h-13z" fill="#f1644b" opacity=".2"/><path d="m18 23h13v18h-13z" fill="#f1644b" opacity=".1"/><path d="m0 23h13v18h-13z" fill="#f1644b" opacity=".1"/><path d="m180 0h13v18h-13z" fill="#00adef" opacity=".6"/><path d="m162 0h13v18h-13z" fill="#00adef" opacity=".5"/><path d="m144 0h13v18h-13z" fill="#9ecc4f" opacity=".5"/><path d="m126 0h13v18h-13z" fill="#ffcd27" opacity=".4"/><path d="m108 0h13v18h-13z" fill="#aa99c9" opacity=".4"/><path d="m90 0h13v18h-13z" fill="#aa99c9" opacity=".3"/><path d="m72 0h13v18h-13z" fill="#aa99c9" opacity=".3"/><path d="m54 0h13v18h-13z" fill="#aa99c9" opacity=".2"/><path d="m36 0h13v18h-13z" fill="#aa99c9" opacity=".2"/><path d="m18 0h13v18h-13z" fill="#aa99c9" opacity=".1"/><path d="m0 0h13v18h-13z" fill="#faab3c" opacity=".1"/><path d="m396 161h13v18h-13z" fill="#ffcd27" transform="matrix(-1 0 0 1 805 0)"/><path d="m414 161h13v18h-13z" fill="#ffcd27" transform="matrix(-1 0 0 1 841 0)"/><path d="m432 161h13v18h-13z" fill="#ffcd27" transform="matrix(-1 0 0 1 877 0)"/><path d="m450 161h13v18h-13z" fill="#ffcd27" transform="matrix(-1 0 0 1 913 0)"/><path d="m468 161h13v18h-13z" fill="#ffcd27" opacity=".9" transform="matrix(-1 0 0 1 949 0)"/><path d="m486 161h13v18h-13z" fill="#f1644b" opacity=".9" transform="matrix(-1 0 0 1 985 0)"/><path d="m504 161h13v18h-13z" fill="#ffcd27" opacity=".8" transform="matrix(-1 0 0 1 1021 0)"/><path d="m522 161h13v18h-13z" fill="#ffcd27" opacity=".8" transform="matrix(-1 0 0 1 1057 0)"/><path d="m540 161h13v18h-13z" fill="#ffcd27" opacity=".7" transform="matrix(-1 0 0 1 1093 0)"/><path d="m558 161h13v18h-13z" fill="#ffcd27" opacity=".7" transform="matrix(-1 0 0 1 1129 0)"/><path d="m576 161h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(-1 0 0 1 1165 0)"/><path d="m396 138h13v18h-13z" fill="#faab3c" transform="matrix(-1 0 0 1 805 0)"/><path d="m414 138h13v18h-13z" fill="#faab3c" transform="matrix(-1 0 0 1 841 0)"/><path d="m432 138h13v18h-13z" fill="#faab3c" transform="matrix(-1 0 0 1 877 0)"/><path d="m450 138h13v18h-13z" fill="#aa99c9" transform="matrix(-1 0 0 1 913 0)"/><path d="m468 138h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(-1 0 0 1 949 0)"/><path d="m486 138h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(-1 0 0 1 985 0)"/><path d="m504 138h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(-1 0 0 1 1021 0)"/><path d="m522 138h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(-1 0 0 1 1057 0)"/><path d="m540 138h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(-1 0 0 1 1093 0)"/><path d="m558 138h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(-1 0 0 1 1129 0)"/><path d="m576 138h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(-1 0 0 1 1165 0)"/><path d="m396 115h13v18h-13z" fill="#f1644b" transform="matrix(-1 0 0 1 805 0)"/><path d="m414 115h13v18h-13z" fill="#f1644b" transform="matrix(-1 0 0 1 841 0)"/><path d="m432 115h13v18h-13z" fill="#f1644b" transform="matrix(-1 0 0 1 877 0)"/><path d="m450 115h13v18h-13z" fill="#f1644b" transform="matrix(-1 0 0 1 913 0)"/><path d="m468 115h13v18h-13z" fill="#f1644b" opacity=".9" transform="matrix(-1 0 0 1 949 0)"/><path d="m486 115h13v18h-13z" fill="#f1644b" opacity=".9" transform="matrix(-1 0 0 1 985 0)"/><path d="m504 115h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(-1 0 0 1 1021 0)"/><path d="m522 115h13v18h-13z" fill="#9ecc4f" opacity=".8" transform="matrix(-1 0 0 1 1057 0)"/><path d="m540 115h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(-1 0 0 1 1093 0)"/><path d="m558 115h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(-1 0 0 1 1129 0)"/><path d="m576 115h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(-1 0 0 1 1165 0)"/><path d="m396 92h13v18h-13z" fill="#333" transform="matrix(-1 0 0 1 805 0)"/><path d="m414 92h13v18h-13z" fill="#333" transform="matrix(-1 0 0 1 841 0)"/><path d="m432 92h13v18h-13z" fill="#9ecc4f" transform="matrix(-1 0 0 1 877 0)"/><path d="m450 92h13v18h-13z" fill="#aa99c9" transform="matrix(-1 0 0 1 913 0)"/><path d="m468 92h13v18h-13z" fill="#f1644b" opacity=".9" transform="matrix(-1 0 0 1 949 0)"/><path d="m486 92h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(-1 0 0 1 985 0)"/><path d="m504 92h13v18h-13z" fill="#faab3c" opacity=".8" transform="matrix(-1 0 0 1 1021 0)"/><path d="m522 92h13v18h-13z" fill="#faab3c" opacity=".8" transform="matrix(-1 0 0 1 1057 0)"/><path d="m540 92h13v18h-13z" fill="#faab3c" opacity=".7" transform="matrix(-1 0 0 1 1093 0)"/><path d="m558 92h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(-1 0 0 1 1129 0)"/><path d="m576 92h13v18h-13z" fill="#ffcd27" opacity=".6" transform="matrix(-1 0 0 1 1165 0)"/><path d="m396 69h13v18h-13z" fill="#f1644b" transform="matrix(-1 0 0 1 805 0)"/><path d="m414 69h13v18h-13z" fill="#333" transform="matrix(-1 0 0 1 841 0)"/><path d="m432 69h13v18h-13z" fill="#9ecc4f" transform="matrix(-1 0 0 1 877 0)"/><path d="m450 69h13v18h-13z" fill="#333" transform="matrix(-1 0 0 1 913 0)"/><path d="m468 69h13v18h-13z" fill="#333" opacity=".9" transform="matrix(-1 0 0 1 949 0)"/><path d="m486 69h13v18h-13z" fill="#00adef" opacity=".9" transform="matrix(-1 0 0 1 985 0)"/><path d="m504 69h13v18h-13z" fill="#00adef" opacity=".8" transform="matrix(-1 0 0 1 1021 0)"/><path d="m522 69h13v18h-13z" fill="#00adef" opacity=".8" transform="matrix(-1 0 0 1 1057 0)"/><path d="m540 69h13v18h-13z" fill="#333" opacity=".7" transform="matrix(-1 0 0 1 1093 0)"/><path d="m558 69h13v18h-13z" fill="#9ecc4f" opacity=".7" transform="matrix(-1 0 0 1 1129 0)"/><path d="m576 69h13v18h-13z" fill="#ffcd27" opacity=".6" transform="matrix(-1 0 0 1 1165 0)"/><path d="m396 46h13v18h-13z" fill="#00adef" transform="matrix(-1 0 0 1 805 0)"/><path d="m414 46h13v18h-13z" fill="#00adef" transform="matrix(-1 0 0 1 841 0)"/><path d="m432 46h13v18h-13z" fill="#9ecc4f" transform="matrix(-1 0 0 1 877 0)"/><path d="m450 46h13v18h-13z" fill="#333" transform="matrix(-1 0 0 1 913 0)"/><path d="m468 46h13v18h-13z" fill="#333" opacity=".9" transform="matrix(-1 0 0 1 949 0)"/><path d="m486 46h13v18h-13z" fill="#ffcd27" opacity=".9" transform="matrix(-1 0 0 1 985 0)"/><path d="m504 46h13v18h-13z" fill="#333" opacity=".8" transform="matrix(-1 0 0 1 1021 0)"/><path d="m522 46h13v18h-13z" fill="#aa99c9" opacity=".8" transform="matrix(-1 0 0 1 1057 0)"/><path d="m540 46h13v18h-13z" fill="#faab3c" opacity=".7" transform="matrix(-1 0 0 1 1093 0)"/><path d="m558 46h13v18h-13z" fill="#faab3c" opacity=".7" transform="matrix(-1 0 0 1 1129 0)"/><path d="m576 46h13v18h-13z" fill="#333" opacity=".6" transform="matrix(-1 0 0 1 1165 0)"/><path d="m396 23h13v18h-13z" fill="#00adef" transform="matrix(-1 0 0 1 805 0)"/><path d="m414 23h13v18h-13z" fill="#00adef" transform="matrix(-1 0 0 1 841 0)"/><path d="m432 23h13v18h-13z" fill="#9ecc4f" transform="matrix(-1 0 0 1 877 0)"/><path d="m450 23h13v18h-13z" fill="#f1644b" transform="matrix(-1 0 0 1 913 0)"/><path d="m468 23h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(-1 0 0 1 949 0)"/><path d="m486 23h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(-1 0 0 1 985 0)"/><path d="m504 23h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(-1 0 0 1 1021 0)"/><path d="m522 23h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(-1 0 0 1 1057 0)"/><path d="m540 23h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(-1 0 0 1 1093 0)"/><path d="m558 23h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(-1 0 0 1 1129 0)"/><path d="m576 23h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(-1 0 0 1 1165 0)"/><path d="m396 0h13v18h-13z" fill="#00adef" transform="matrix(-1 0 0 1 805 0)"/><path d="m414 0h13v18h-13z" fill="#00adef" transform="matrix(-1 0 0 1 841 0)"/><path d="m432 0h13v18h-13z" fill="#9ecc4f" transform="matrix(-1 0 0 1 877 0)"/><path d="m450 0h13v18h-13z" fill="#ffcd27" transform="matrix(-1 0 0 1 913 0)"/><path d="m468 0h13v18h-13z" fill="#aa99c9" opacity=".9" transform="matrix(-1 0 0 1 949 0)"/><path d="m486 0h13v18h-13z" fill="#aa99c9" opacity=".9" transform="matrix(-1 0 0 1 985 0)"/><path d="m504 0h13v18h-13z" fill="#aa99c9" opacity=".8" transform="matrix(-1 0 0 1 1021 0)"/><path d="m522 0h13v18h-13z" fill="#aa99c9" opacity=".8" transform="matrix(-1 0 0 1 1057 0)"/><path d="m540 0h13v18h-13z" fill="#aa99c9" opacity=".7" transform="matrix(-1 0 0 1 1093 0)"/><path d="m558 0h13v18h-13z" fill="#aa99c9" opacity=".7" transform="matrix(-1 0 0 1 1129 0)"/><path d="m576 0h13v18h-13z" fill="#faab3c" opacity=".6" transform="matrix(-1 0 0 1 1165 0)"/><path d="m378 0h13v18h-13z" fill="#ffcd27" transform="matrix(1 0 0 -1 0 18)"/><path d="m360 0h13v18h-13z" fill="#ffcd27" transform="matrix(1 0 0 -1 0 18)"/><path d="m342 0h13v18h-13z" fill="#ffcd27" transform="matrix(1 0 0 -1 0 18)"/><path d="m324 0h13v18h-13z" fill="#ffcd27" transform="matrix(1 0 0 -1 0 18)"/><path d="m306 0h13v18h-13z" fill="#ffcd27" opacity=".9" transform="matrix(1 0 0 -1 0 18)"/><path d="m288 0h13v18h-13z" fill="#f1644b" opacity=".9" transform="matrix(1 0 0 -1 0 18)"/><path d="m270 0h13v18h-13z" fill="#ffcd27" opacity=".8" transform="matrix(1 0 0 -1 0 18)"/><path d="m252 0h13v18h-13z" fill="#ffcd27" opacity=".8" transform="matrix(1 0 0 -1 0 18)"/><path d="m234 0h13v18h-13z" fill="#ffcd27" opacity=".7" transform="matrix(1 0 0 -1 0 18)"/><path d="m216 0h13v18h-13z" fill="#ffcd27" opacity=".7" transform="matrix(1 0 0 -1 0 18)"/><path d="m198 0h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(1 0 0 -1 0 18)"/><path d="m378 23h13v18h-13z" fill="#faab3c" transform="matrix(1 0 0 -1 0 64)"/><path d="m360 23h13v18h-13z" fill="#faab3c" transform="matrix(1 0 0 -1 0 64)"/><path d="m342 23h13v18h-13z" fill="#faab3c" transform="matrix(1 0 0 -1 0 64)"/><path d="m324 23h13v18h-13z" fill="#aa99c9" transform="matrix(1 0 0 -1 0 64)"/><path d="m306 23h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(1 0 0 -1 0 64)"/><path d="m288 23h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(1 0 0 -1 0 64)"/><path d="m270 23h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(1 0 0 -1 0 64)"/><path d="m252 23h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(1 0 0 -1 0 64)"/><path d="m234 23h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(1 0 0 -1 0 64)"/><path d="m216 23h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(1 0 0 -1 0 64)"/><path d="m198 23h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(1 0 0 -1 0 64)"/><path d="m378 46h13v18h-13z" fill="#f1644b" transform="matrix(1 0 0 -1 0 110)"/><path d="m360 46h13v18h-13z" fill="#f1644b" transform="matrix(1 0 0 -1 0 110)"/><path d="m342 46h13v18h-13z" fill="#f1644b" transform="matrix(1 0 0 -1 0 110)"/><path d="m324 46h13v18h-13z" fill="#f1644b" transform="matrix(1 0 0 -1 0 110)"/><path d="m306 46h13v18h-13z" fill="#f1644b" opacity=".9" transform="matrix(1 0 0 -1 0 110)"/><path d="m288 46h13v18h-13z" fill="#f1644b" opacity=".9" transform="matrix(1 0 0 -1 0 110)"/><path d="m270 46h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(1 0 0 -1 0 110)"/><path d="m252 46h13v18h-13z" fill="#9ecc4f" opacity=".8" transform="matrix(1 0 0 -1 0 110)"/><path d="m234 46h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(1 0 0 -1 0 110)"/><path d="m216 46h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(1 0 0 -1 0 110)"/><path d="m198 46h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(1 0 0 -1 0 110)"/><path d="m378 69h13v18h-13z" fill="#333" transform="matrix(1 0 0 -1 0 156)"/><path d="m360 69h13v18h-13z" fill="#333" transform="matrix(1 0 0 -1 0 156)"/><path d="m342 69h13v18h-13z" fill="#9ecc4f" transform="matrix(1 0 0 -1 0 156)"/><path d="m324 69h13v18h-13z" fill="#aa99c9" transform="matrix(1 0 0 -1 0 156)"/><path d="m306 69h13v18h-13z" fill="#f1644b" opacity=".9" transform="matrix(1 0 0 -1 0 156)"/><path d="m288 69h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(1 0 0 -1 0 156)"/><path d="m270 69h13v18h-13z" fill="#faab3c" opacity=".8" transform="matrix(1 0 0 -1 0 156)"/><path d="m252 69h13v18h-13z" fill="#faab3c" opacity=".8" transform="matrix(1 0 0 -1 0 156)"/><path d="m234 69h13v18h-13z" fill="#faab3c" opacity=".7" transform="matrix(1 0 0 -1 0 156)"/><path d="m216 69h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(1 0 0 -1 0 156)"/><path d="m198 69h13v18h-13z" fill="#ffcd27" opacity=".6" transform="matrix(1 0 0 -1 0 156)"/><path d="m378 92h13v18h-13z" fill="#f1644b" transform="matrix(1 0 0 -1 0 202)"/><path d="m360 92h13v18h-13z" fill="#333" transform="matrix(1 0 0 -1 0 202)"/><path d="m342 92h13v18h-13z" fill="#9ecc4f" transform="matrix(1 0 0 -1 0 202)"/><path d="m324 92h13v18h-13z" fill="#333" transform="matrix(1 0 0 -1 0 202)"/><path d="m306 92h13v18h-13z" fill="#333" opacity=".9" transform="matrix(1 0 0 -1 0 202)"/><path d="m288 92h13v18h-13z" fill="#00adef" opacity=".9" transform="matrix(1 0 0 -1 0 202)"/><path d="m270 92h13v18h-13z" fill="#00adef" opacity=".8" transform="matrix(1 0 0 -1 0 202)"/><path d="m252 92h13v18h-13z" fill="#00adef" opacity=".8" transform="matrix(1 0 0 -1 0 202)"/><path d="m234 92h13v18h-13z" fill="#333" opacity=".7" transform="matrix(1 0 0 -1 0 202)"/><path d="m216 92h13v18h-13z" fill="#9ecc4f" opacity=".7" transform="matrix(1 0 0 -1 0 202)"/><path d="m198 92h13v18h-13z" fill="#ffcd27" opacity=".6" transform="matrix(1 0 0 -1 0 202)"/><path d="m378 115h13v18h-13z" fill="#00adef" transform="matrix(1 0 0 -1 0 248)"/><path d="m360 115h13v18h-13z" fill="#00adef" transform="matrix(1 0 0 -1 0 248)"/><path d="m342 115h13v18h-13z" fill="#9ecc4f" transform="matrix(1 0 0 -1 0 248)"/><path d="m324 115h13v18h-13z" fill="#333" transform="matrix(1 0 0 -1 0 248)"/><path d="m306 115h13v18h-13z" fill="#333" opacity=".9" transform="matrix(1 0 0 -1 0 248)"/><path d="m288 115h13v18h-13z" fill="#ffcd27" opacity=".9" transform="matrix(1 0 0 -1 0 248)"/><path d="m270 115h13v18h-13z" fill="#333" opacity=".8" transform="matrix(1 0 0 -1 0 248)"/><path d="m252 115h13v18h-13z" fill="#aa99c9" opacity=".8" transform="matrix(1 0 0 -1 0 248)"/><path d="m234 115h13v18h-13z" fill="#faab3c" opacity=".7" transform="matrix(1 0 0 -1 0 248)"/><path d="m216 115h13v18h-13z" fill="#faab3c" opacity=".7" transform="matrix(1 0 0 -1 0 248)"/><path d="m198 115h13v18h-13z" fill="#333" opacity=".6" transform="matrix(1 0 0 -1 0 248)"/><path d="m378 138h13v18h-13z" fill="#00adef" transform="matrix(1 0 0 -1 0 294)"/><path d="m360 138h13v18h-13z" fill="#00adef" transform="matrix(1 0 0 -1 0 294)"/><path d="m342 138h13v18h-13z" fill="#9ecc4f" transform="matrix(1 0 0 -1 0 294)"/><path d="m324 138h13v18h-13z" fill="#f1644b" transform="matrix(1 0 0 -1 0 294)"/><path d="m306 138h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(1 0 0 -1 0 294)"/><path d="m288 138h13v18h-13z" fill="#faab3c" opacity=".9" transform="matrix(1 0 0 -1 0 294)"/><path d="m270 138h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(1 0 0 -1 0 294)"/><path d="m252 138h13v18h-13z" fill="#f1644b" opacity=".8" transform="matrix(1 0 0 -1 0 294)"/><path d="m234 138h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(1 0 0 -1 0 294)"/><path d="m216 138h13v18h-13z" fill="#f1644b" opacity=".7" transform="matrix(1 0 0 -1 0 294)"/><path d="m198 138h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(1 0 0 -1 0 294)"/><path d="m378 161h13v18h-13z" fill="#00adef" transform="matrix(1 0 0 -1 0 340)"/><path d="m360 161h13v18h-13z" fill="#00adef" transform="matrix(1 0 0 -1 0 340)"/><path d="m342 161h13v18h-13z" fill="#9ecc4f" transform="matrix(1 0 0 -1 0 340)"/><path d="m324 161h13v18h-13z" fill="#ffcd27" transform="matrix(1 0 0 -1 0 340)"/><path d="m306 161h13v18h-13z" fill="#aa99c9" opacity=".9" transform="matrix(1 0 0 -1 0 340)"/><path d="m288 161h13v18h-13z" fill="#aa99c9" opacity=".9" transform="matrix(1 0 0 -1 0 340)"/><path d="m270 161h13v18h-13z" fill="#aa99c9" opacity=".8" transform="matrix(1 0 0 -1 0 340)"/><path d="m252 161h13v18h-13z" fill="#aa99c9" opacity=".8" transform="matrix(1 0 0 -1 0 340)"/><path d="m234 161h13v18h-13z" fill="#aa99c9" opacity=".7" transform="matrix(1 0 0 -1 0 340)"/><path d="m216 161h13v18h-13z" fill="#aa99c9" opacity=".7" transform="matrix(1 0 0 -1 0 340)"/><path d="m198 161h13v18h-13z" fill="#faab3c" opacity=".6" transform="matrix(1 0 0 -1 0 340)"/><path d="m594 0h13v18h-13z" fill="#ffcd27" opacity=".6" transform="matrix(-1 0 0 -1 1201 18)"/><path d="m612 0h13v18h-13z" fill="#ffcd27" opacity=".5" transform="matrix(-1 0 0 -1 1237 18)"/><path d="m630 0h13v18h-13z" fill="#ffcd27" opacity=".5" transform="matrix(-1 0 0 -1 1273 18)"/><path d="m648 0h13v18h-13z" fill="#ffcd27" opacity=".4" transform="matrix(-1 0 0 -1 1309 18)"/><path d="m666 0h13v18h-13z" fill="#ffcd27" opacity=".4" transform="matrix(-1 0 0 -1 1345 18)"/><path d="m684 0h13v18h-13z" fill="#f1644b" opacity=".3" transform="matrix(-1 0 0 -1 1381 18)"/><path d="m702 0h13v18h-13z" fill="#ffcd27" opacity=".3" transform="matrix(-1 0 0 -1 1417 18)"/><path d="m720 0h13v18h-13z" fill="#ffcd27" opacity=".2" transform="matrix(-1 0 0 -1 1453 18)"/><path d="m738 0h13v18h-13z" fill="#ffcd27" opacity=".2" transform="matrix(-1 0 0 -1 1489 18)"/><path d="m756 0h13v18h-13z" fill="#ffcd27" opacity=".1" transform="matrix(-1 0 0 -1 1525 18)"/><path d="m774 0h13v18h-13z" fill="#f1644b" opacity=".1" transform="matrix(-1 0 0 -1 1561 18)"/><path d="m594 23h13v18h-13z" fill="#faab3c" opacity=".6" transform="matrix(-1 0 0 -1 1201 64)"/><path d="m612 23h13v18h-13z" fill="#faab3c" opacity=".5" transform="matrix(-1 0 0 -1 1237 64)"/><path d="m630 23h13v18h-13z" fill="#faab3c" opacity=".5" transform="matrix(-1 0 0 -1 1273 64)"/><path d="m648 23h13v18h-13z" fill="#aa99c9" opacity=".4" transform="matrix(-1 0 0 -1 1309 64)"/><path d="m666 23h13v18h-13z" fill="#faab3c" opacity=".4" transform="matrix(-1 0 0 -1 1345 64)"/><path d="m684 23h13v18h-13z" fill="#faab3c" opacity=".3" transform="matrix(-1 0 0 -1 1381 64)"/><path d="m702 23h13v18h-13z" fill="#f1644b" opacity=".3" transform="matrix(-1 0 0 -1 1417 64)"/><path d="m720 23h13v18h-13z" fill="#f1644b" opacity=".2" transform="matrix(-1 0 0 -1 1453 64)"/><path d="m738 23h13v18h-13z" fill="#f1644b" opacity=".2" transform="matrix(-1 0 0 -1 1489 64)"/><path d="m756 23h13v18h-13z" fill="#f1644b" opacity=".1" transform="matrix(-1 0 0 -1 1525 64)"/><path d="m774 23h13v18h-13z" fill="#f1644b" opacity=".1" transform="matrix(-1 0 0 -1 1561 64)"/><path d="m594 46h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(-1 0 0 -1 1201 110)"/><path d="m612 46h13v18h-13z" fill="#f1644b" opacity=".5" transform="matrix(-1 0 0 -1 1237 110)"/><path d="m630 46h13v18h-13z" fill="#f1644b" opacity=".5" transform="matrix(-1 0 0 -1 1273 110)"/><path d="m648 46h13v18h-13z" fill="#f1644b" opacity=".4" transform="matrix(-1 0 0 -1 1309 110)"/><path d="m666 46h13v18h-13z" fill="#f1644b" opacity=".4" transform="matrix(-1 0 0 -1 1345 110)"/><path d="m684 46h13v18h-13z" fill="#f1644b" opacity=".3" transform="matrix(-1 0 0 -1 1381 110)"/><path d="m702 46h13v18h-13z" fill="#f1644b" opacity=".3" transform="matrix(-1 0 0 -1 1417 110)"/><path d="m720 46h13v18h-13z" fill="#9ecc4f" opacity=".2" transform="matrix(-1 0 0 -1 1453 110)"/><path d="m738 46h13v18h-13z" fill="#f1644b" opacity=".2" transform="matrix(-1 0 0 -1 1489 110)"/><path d="m756 46h13v18h-13z" fill="#f1644b" opacity=".1" transform="matrix(-1 0 0 -1 1525 110)"/><path d="m774 46h13v18h-13z" fill="#f1644b" opacity=".1" transform="matrix(-1 0 0 -1 1561 110)"/><path d="m594 69h13v18h-13z" fill="#333" opacity=".6" transform="matrix(-1 0 0 -1 1201 156)"/><path d="m612 69h13v18h-13z" fill="#333" opacity=".5" transform="matrix(-1 0 0 -1 1237 156)"/><path d="m630 69h13v18h-13z" fill="#9ecc4f" opacity=".5" transform="matrix(-1 0 0 -1 1273 156)"/><path d="m648 69h13v18h-13z" fill="#aa99c9" opacity=".4" transform="matrix(-1 0 0 -1 1309 156)"/><path d="m666 69h13v18h-13z" fill="#f1644b" opacity=".4" transform="matrix(-1 0 0 -1 1345 156)"/><path d="m684 69h13v18h-13z" fill="#faab3c" opacity=".3" transform="matrix(-1 0 0 -1 1381 156)"/><path d="m702 69h13v18h-13z" fill="#faab3c" opacity=".3" transform="matrix(-1 0 0 -1 1417 156)"/><path d="m720 69h13v18h-13z" fill="#faab3c" opacity=".2" transform="matrix(-1 0 0 -1 1453 156)"/><path d="m738 69h13v18h-13z" fill="#faab3c" opacity=".2" transform="matrix(-1 0 0 -1 1489 156)"/><path d="m756 69h13v18h-13z" fill="#f1644b" opacity=".1" transform="matrix(-1 0 0 -1 1525 156)"/><path d="m774 69h13v18h-13z" fill="#ffcd27" opacity=".1" transform="matrix(-1 0 0 -1 1561 156)"/><path d="m594 92h13v18h-13z" fill="#f1644b" opacity=".6" transform="matrix(-1 0 0 -1 1201 202)"/><path d="m612 92h13v18h-13z" fill="#333" opacity=".5" transform="matrix(-1 0 0 -1 1237 202)"/><path d="m630 92h13v18h-13z" fill="#9ecc4f" opacity=".5" transform="matrix(-1 0 0 -1 1273 202)"/><path d="m648 92h13v18h-13z" fill="#333" opacity=".4" transform="matrix(-1 0 0 -1 1309 202)"/><path d="m666 92h13v18h-13z" fill="#333" opacity=".4" transform="matrix(-1 0 0 -1 1345 202)"/><path d="m684 92h13v18h-13z" fill="#00adef" opacity=".3" transform="matrix(-1 0 0 -1 1381 202)"/><path d="m702 92h13v18h-13z" fill="#00adef" opacity=".3" transform="matrix(-1 0 0 -1 1417 202)"/><path d="m720 92h13v18h-13z" fill="#00adef" opacity=".2" transform="matrix(-1 0 0 -1 1453 202)"/><path d="m738 92h13v18h-13z" fill="#333" opacity=".2" transform="matrix(-1 0 0 -1 1489 202)"/><path d="m756 92h13v18h-13z" fill="#9ecc4f" opacity=".1" transform="matrix(-1 0 0 -1 1525 202)"/><path d="m774 92h13v18h-13z" fill="#ffcd27" opacity=".1" transform="matrix(-1 0 0 -1 1561 202)"/><path d="m594 115h13v18h-13z" fill="#00adef" opacity=".6" transform="matrix(-1 0 0 -1 1201 248)"/><path d="m612 115h13v18h-13z" fill="#00adef" opacity=".5" transform="matrix(-1 0 0 -1 1237 248)"/><path d="m630 115h13v18h-13z" fill="#9ecc4f" opacity=".5" transform="matrix(-1 0 0 -1 1273 248)"/><path d="m648 115h13v18h-13z" fill="#333" opacity=".4" transform="matrix(-1 0 0 -1 1309 248)"/><path d="m666 115h13v18h-13z" fill="#333" opacity=".4" transform="matrix(-1 0 0 -1 1345 248)"/><path d="m684 115h13v18h-13z" fill="#ffcd27" opacity=".3" transform="matrix(-1 0 0 -1 1381 248)"/><path d="m702 115h13v18h-13z" fill="#333" opacity=".3" transform="matrix(-1 0 0 -1 1417 248)"/><path d="m720 115h13v18h-13z" fill="#aa99c9" opacity=".2" transform="matrix(-1 0 0 -1 1453 248)"/><path d="m738 115h13v18h-13z" fill="#faab3c" opacity=".2" transform="matrix(-1 0 0 -1 1489 248)"/><path d="m756 115h13v18h-13z" fill="#faab3c" opacity=".1" transform="matrix(-1 0 0 -1 1525 248)"/><path d="m774 115h13v18h-13z" fill="#333" opacity=".1" transform="matrix(-1 0 0 -1 1561 248)"/><path d="m594 138h13v18h-13z" fill="#00adef" opacity=".6" transform="matrix(-1 0 0 -1 1201 294)"/><path d="m612 138h13v18h-13z" fill="#00adef" opacity=".5" transform="matrix(-1 0 0 -1 1237 294)"/><path d="m630 138h13v18h-13z" fill="#9ecc4f" opacity=".5" transform="matrix(-1 0 0 -1 1273 294)"/><path d="m648 138h13v18h-13z" fill="#f1644b" opacity=".4" transform="matrix(-1 0 0 -1 1309 294)"/><path d="m666 138h13v18h-13z" fill="#faab3c" opacity=".4" transform="matrix(-1 0 0 -1 1345 294)"/><path d="m684 138h13v18h-13z" fill="#faab3c" opacity=".3" transform="matrix(-1 0 0 -1 1381 294)"/><path d="m702 138h13v18h-13z" fill="#f1644b" opacity=".3" transform="matrix(-1 0 0 -1 1417 294)"/><path d="m720 138h13v18h-13z" fill="#f1644b" opacity=".2" transform="matrix(-1 0 0 -1 1453 294)"/><path d="m738 138h13v18h-13z" fill="#f1644b" opacity=".2" transform="matrix(-1 0 0 -1 1489 294)"/><path d="m756 138h13v18h-13z" fill="#f1644b" opacity=".1" transform="matrix(-1 0 0 -1 1525 294)"/><path d="m774 138h13v18h-13z" fill="#f1644b" opacity=".1" transform="matrix(-1 0 0 -1 1561 294)"/><path d="m594 161h13v18h-13z" fill="#00adef" opacity=".6" transform="matrix(-1 0 0 -1 1201 340)"/><path d="m612 161h13v18h-13z" fill="#00adef" opacity=".5" transform="matrix(-1 0 0 -1 1237 340)"/><path d="m630 161h13v18h-13z" fill="#9ecc4f" opacity=".5" transform="matrix(-1 0 0 -1 1273 340)"/><path d="m648 161h13v18h-13z" fill="#ffcd27" opacity=".4" transform="matrix(-1 0 0 -1 1309 340)"/><path d="m666 161h13v18h-13z" fill="#aa99c9" opacity=".4" transform="matrix(-1 0 0 -1 1345 340)"/><path d="m684 161h13v18h-13z" fill="#aa99c9" opacity=".3" transform="matrix(-1 0 0 -1 1381 340)"/><path d="m702 161h13v18h-13z" fill="#aa99c9" opacity=".3" transform="matrix(-1 0 0 -1 1417 340)"/><path d="m720 161h13v18h-13z" fill="#aa99c9" opacity=".2" transform="matrix(-1 0 0 -1 1453 340)"/><path d="m738 161h13v18h-13z" fill="#aa99c9" opacity=".2" transform="matrix(-1 0 0 -1 1489 340)"/><path d="m756 161h13v18h-13z" fill="#aa99c9" opacity=".1" transform="matrix(-1 0 0 -1 1525 340)"/><path d="m774 161h13v18h-13z" fill="#faab3c" opacity=".1" transform="matrix(-1 0 0 -1 1561 340)"/></g><g transform="translate(229)"><path d="m0 163.414428c0 90.282661 73.1123182 163.408804 163.387152 163.408804 36.142571 0 69.465987-11.874563 96.503984-31.804247l97.84885 97.516523c4.912418 4.618198 11.51858 7.464492 18.788734 7.464492 15.170539 0 27.47128-12.296442 27.47128-27.456054 0-8.364506-3.736364-15.823372-9.616636-20.857826l-96.093209-96.172128c17.888406-26.241035 28.422252-57.938405 28.422252-92.099564 0-90.2320345-73.112318-163.414428-163.325255-163.414428-90.2748338 0-163.387152 73.1823935-163.387152 163.414428z" fill="#999"/><g transform="translate(36 36)"><use fill="#fff" xlink:href="#a"/><path d="m135.660763 148.70091c.364228-.579415 1.490837-1.136024 2.636245-1.577175l.457403-.170083.448833-.15645.429688-.141498.759638-.232874.836301-.231431 18.280829-.001215.19491-.011051.202794-.017881.247815-.029781c.621919-.085699 1.518677-.293004 2.040439-.792877.397637-.380753.702259-.841071.925774-1.260385l.137125-.272145c.04179-.087808.079706-.172268.113878-.252057l.128943-.323055.119178-.358057v-45.185461h-23.10923c-3.36553 0-5.599705 1.3581721-7.076583 2.93031v48.068902zm-8.205086 0 2.160788-.014264v-48.137167c-1.476878-1.5446282-3.696783-2.862045-7.010333-2.862045h-23.1092292l.0007678 45.713737.0112285.168178.0209214.173899.0370444.211161c.0932827.452634.3109425 1.066293.8188151 1.465695.526089.412166 1.208439.604335 1.713672.693785l.256013.039309.208859.023165.228168.014813 19.094157.000223.237682.060474.480012.132689.315282.093319.355116.110754.387189.127778.411498.144393.428047.160596c1.084331.421403 2.251026.990863 2.954302 1.679508zm5.548742 8.747628c.251851 0 .525983-.01408.812699-.039079l.438298-.045293c.074219-.008782.148921-.018148.223954-.028048l.452973-.065416.453665-.075869.447082-.08395.433227-.089662.412098-.093003.383696-.093972.34802-.092573.305071-.088801.254848-.08266.197352-.074149c.110787-.046068.178394-.084881.193124-.113278.075334-.143783.342864-.272994.772162-.389029l.276747-.068051c.049279-.011083.100115-.022036.152477-.032861l.332246-.063435.367419-.06044.401131-.057513.433384-.054653.464175-.05186.493506-.049135 1.069163-.090361.868004-.061115.919211-.055662 1.296751-.066125 1.019525-.043819 1.412611-.051157 1.834904-.053019 2.657035-.05571 1.374969-.02089 2.477068-.026383 1.957947-.011997 1.910166-.005129 6.045147.020483 5.014577.056935v-53.988617l-3.71615-1.3204734-.588101 50.8117374-.77828.02962-1.822742.039073-5.842498.076788-3.480825.060896-1.809182.042629-.912892.02518c-.609594.017723-1.220619.037372-1.829689.059259l-1.291501.050048-1.505858.068618-1.475684.080037-1.079809.068179-1.051134.075682-1.348236.113376-.964719.094983-.919324.104025-.585187.074603-.561296.078963-.53592.083462-.509057.088098c-.165043.030153-.325362.061102-.480708.092869l-.450874.097779c-1.306381.300838-2.18993.669802-2.470085 1.123534-.611907.992257-7.826645.987033-9.518061-.529048l-.106623-.105716c-.228962-.252838-.78901-.474074-1.603516-.667611l-.428103-.094479c-.074767-.015367-.151211-.030547-.22929-.045542l-.487727-.087757c-.084437-.014261-.17042-.028341-.257904-.042242l-.542561-.08128-.576456-.077098-.608224-.073023-.637861-.069057-1.007709-.096387-1.062421-.088074-1.109951-.080126-1.541453-.095106-1.192916-.063006-2.037053-.090241-1.65446-.059876-2.071158-.060872-1.231568-.029723-3.180948-.0575-2.57634-.028621-3.1568948-.015367-3.5804204.010051-.5238893-51.2681974-3.3104917 1.4162484v54.074204l6.091503-.110017 4.8697032-.042899 1.42012-.004518 1.451867-.000435 2.462799.010003 2.199758.022091 1.996082.032898 1.566274.036091 1.898382.058605 1.097614.042942 1.059883.049177 1.34505.075837.950618.065003.603014.047387.576542.050742.548454.054194.518747.057743.487425.06139.454485.065134.419927.068975.383754.072913c.182564.037458.350956.076428.504267.116967l.286244.083185c.309863.099526.534315.207787.661802.32548l.048667.051019c.714453.863732 2.602457 1.171499 4.492467 1.281163l.565891.027314c.093935.003681.187582.006956.280794.00987l.552892.013511 1.046396.010012z" fill="#f9a72b" mask="url(#b)"/><path d="m226.413899 74.9897567c.315665-.5021599 1.203961-.98719 2.180847-1.394777l.455398-.1823985c.076361-.02941.152805-.058307.229112-.0866633l.45444-.163431.440583-.1491388.416149-.133529.555278-.1681423.836301-.231431 18.280829-.0012149.289969-.0186911.226726-.0234574c.620722-.0741415 1.610232-.2738639 2.169263-.8094424.441819-.4230583.768804-.9443454.997292-1.3984719l.125403-.2630934.102548-.2390362.080477-.2070401.119178-.3580573v-45.1854607h-23.10923c-3.36553 0-5.599704 1.3581721-7.076583 2.9303099v48.068902zm-8.205086 0 2.160789-.0142644v-48.1371672c-1.476879-1.5446279-3.696784-2.8620447-7.010333-2.8620447h-23.10923l.000768 45.713737.011228.1681782.020922.1738987.037044.2111608c.093283.452634.310943 1.0662932.818815 1.4656956.526089.4121654 1.208439.6043343 1.713672.6937848l.256013.0393092.208859.0231646.228169.0148134 19.094156.0002231.450008.1176898.419863.1199271.336169.1020876.372123.1193177.400314.136137.420742.1525458.43341.1685439c1.020028.4116141 2.080108.9505261 2.736499 1.593262zm5.548743 8.7476273c.125925 0 .257421-.00352.393275-.0101649l.419423-.0289141.438298-.0452929.4499-.0593011c.075546-.0109191.151272-.0223232.227027-.0341628l.453665-.0758686.447082-.0839505.433227-.0896618.412098-.0930025.383696-.0939728.34802-.0925724.305071-.0888015.254848-.0826602.197353-.0741482c.110786-.046068.178393-.084881.193123-.1132782.075334-.1437836.342864-.2729937.772162-.3890291l.276747-.0680514.314112-.0649565.350015-.0619288.384458-.0589682.41744-.0560748.684807-.0788337.493506-.0491347.79206-.0687384.84984-.0629831 1.214478-.0754167 1.296751-.0661249 1.019525-.0438192 1.774055-.0627038 2.224247-.0594956 2.291057-.0440264.99016-.0145499 2.477069-.0263828 1.957947-.0119975 1.910165-.0051283 2.721728.0027087 3.594993.0198972 4.743003.054812v-53.9886171l-3.71615-1.3204735-.588101 50.8117373-.564488.0228292-.927606.0251154-3.067844.0477809-3.883582.0497561-3.480825.0608956-1.809182.0426292-.912892.0251796c-.609594.0177232-1.220619.0373723-1.829688.0592595l-1.670145.0661913-1.869571.0909968-1.096141.0634357-.716996.0462582-1.051134.0756814-1.348236.1133765-.964718.0949828-.919325.1040253-.585186.0746022-.561297.0789636-.53592.083462-.509057.0880973-.480708.0928697-.450873.0977791c-1.306382.3008381-2.189931.6698015-2.470086 1.1235341-.611907.992257-7.826644.9870322-9.518061-.5290483l-.106623-.1057164c-.248042-.2739072-.88465-.5107264-1.812399-.7154203l-.44851-.0922111-.487727-.0877573-.524814-.083412-.559775-.0791751-.592606-.0750466-.623308-.0710266-.651883-.067115-.678328-.0633117-1.062422-.0880739-1.109951-.0801266-1.541453-.0951055-1.192916-.063006-1.625998-.0736063-2.065515-.0765106-2.071158-.0608726-2.446011-.0547904-3.468741-.0509412-3.949414-.0249683-3.862005.0095403-.523889-51.2681973-3.310492 1.4162486v54.0742034l6.387111-.1137516 4.574095-.0391642 1.420121-.0045179 1.451867-.0004351c.813534.0010052 1.637073.0041829 2.462798.0100033l2.199758.0220909 2.390806.0410703 1.940044.049489 1.129888.0370348 1.097614.0429419 1.059883.0491773.682931.0364111 1.301809.0819627.913942.069853.576543.0507419.548453.0541941.518748.0577433.487424.0613899.454485.0651338.419927.0689749.383754.0729133c.730255.1498297 1.233764.323873 1.452314.5256313l.048667.0510189c.750174.9069191 2.794191 1.2008859 4.775836 1.2961718l.563316.0221761.552892.0135106.794417.0087154z" fill="#f9a72b" mask="url(#b)"/><path d="m64.7035886 87.307553c6.5290026 0 8.8607892 2.6703715 8.8607892 8.9012382-.2664899 6.1037058-.2284199 12.2074128-.1468413 18.3111188l.0963646 6.866669c.0290624 2.28889.0504767 4.57778.0504767 6.86667v31.154333l-.0061309.560469c-.0041257.183941-.0103717.364997-.0188531.54315l-.0322656.525739c-.3593512 4.739178-2.4962479 7.271881-8.8035396 7.271881-15.8561492-.445062-31.245941-.445062-47.1020902 0-6.5290026 0-8.86078924-2.670372-8.86078924-8.901239v-63.1987908l.00613096-.5604681c.00412565-.1839416.01037166-.364997.01885309-.5431504l.03226559-.5257385c.35935121-4.7391787 2.4962479-7.2718812 8.8035396-7.2718812 15.8561492.4450619 31.245941.4450619 47.1020902 0zm-23.1458972 39.690621c-9.4279018 0-16.6110651 7.629444-16.6110651 16.60526 0 9.424607 7.632111 16.60526 16.6110651 16.60526 9.4279018 0 16.6110651-7.629444 16.6110651-16.60526s-7.632111-16.60526-16.6110651-16.60526zm0 12.15019c2.4613273 0 4.4566273 1.994603 4.4566273 4.45507s-1.9953 4.45507-4.4566273 4.45507c-2.4613272 0-4.4566272-1.994603-4.4566272-4.45507s1.9953-4.45507 4.4566272-4.45507zm-.4051479-42.9306715c-6.3527195 0-11.344142 4.9896785-11.344142 11.3401775s4.9914225 11.340177 11.344142 11.340177 11.344142-4.989678 11.344142-11.340177-5.4451882-11.3401775-11.344142-11.3401775z" fill="#00adef" mask="url(#b)"/><path d="m155.456725 173.978909c6.529002 0 8.860789 2.670372 8.860789 8.901239-.26649 6.103706-.22842 12.207412-.146841 18.311118l.096364 6.86667c.029063 2.28889.050477 4.577779.050477 6.866669v31.154334l-.006131.560468c-.121707 5.426278-2.088654 8.34077-8.854658 8.34077-15.856149-.445062-31.245941-.445062-47.10209 0-6.529003 0-8.8607897-2.670371-8.8607897-8.901238v-63.198791l.006131-.560468c.1217068-5.426279 2.0886547-8.340771 8.8546587-8.340771 15.856149.445062 31.245941.445062 47.10209 0zm-23.145897 39.690622c-9.427902 0-16.611066 7.629443-16.611066 16.605259 0 9.424607 7.632111 16.60526 16.611066 16.60526 9.427901 0 16.611065-7.629443 16.611065-16.60526 0-8.975816-7.632111-16.605259-16.611065-16.605259zm0 12.15019c2.461327 0 4.456627 1.994602 4.456627 4.455069 0 2.460468-1.9953 4.45507-4.456627 4.45507-2.461328 0-4.456628-1.994602-4.456628-4.45507 0-2.460467 1.9953-4.455069 4.456628-4.455069zm-.405148-42.930672c-6.35272 0-11.344142 4.989678-11.344142 11.340177 0 6.3505 4.991422 11.340178 11.344142 11.340178 6.352719 0 11.344142-4.989678 11.344142-11.340178 0-6.350499-5.445189-11.340177-11.344142-11.340177z" fill="#00adef" mask="url(#b)"/><path d="m76.3922457 254.20156c2.6910121 0 4.1133203-1.34856 4.1970497-3.976974l.0039259-.250162v-70.456031c-.0048728-2.573165-1.3800402-4.031583-3.8734941-4.117609l-.2370299-.004036h-70.57739562c-2.70601122 0-4.14569964 1.456767-4.14569964 4.17439-.00628136 23.489112-.00628136 46.974455 0 70.457287 0 2.637707 1.35375661 4.083911 3.91006489 4.169138l.24317239.003997zm-18.8440893-48.024331-33.2284107-.002512c-1.7361688 0-2.9497281-1.087552-2.9560095-2.808044-.0201003-4.486675-.0189836-8.972233-.0152613-13.457792l.0052112-6.728477.07412-.351633h39.376609v20.051894c-.0012563 2.274315-1.0150683 3.296564-3.2562587 3.296564zm-2.9399291-4.284026v-16.27937h-9.9182724v16.27937zm13.8946264 45.346016h-55.6867964l-.0309357-.517247c-.009579-.164357-.0168026-.322906-.0168026-.482397l-.0012563-24.413404c0-1.46807.3442187-2.835673 1.59421-3.705965.6030108-.42196 1.4271257-.740942 2.1494824-.740942 8.05522-.020721 16.1098119-.028256 24.1637757-.030297l24.1600068.002669c2.0703373 0 3.732386 1.609978 3.7449487 3.850385l.0088974 2.052071.0110909 3.997474.0038391 5.832642-.0137772 13.808401z" fill="#9ecc4f" mask="url(#b)"/><path d="m257.898518 254.20156c2.691012 0 4.11332-1.34856 4.19705-3.976974l.003926-.250162v-70.456031c-.004873-2.573165-1.380041-4.031583-3.873495-4.117609l-.237029-.004036h-70.577396c-2.706011 0-4.1457 1.456767-4.1457 4.17439-.006281 23.489112-.006281 46.974455 0 70.457287 0 2.637707 1.353757 4.083911 3.910065 4.169138l.243173.003997zm-18.844089-48.024331-33.228411-.002512c-1.736169 0-2.949728-1.087552-2.956009-2.808044-.020101-4.486675-.018984-8.972233-.015262-13.457792l.005211-6.728477.07412-.351633h39.376609v20.051894c-.001256 2.274315-1.015068 3.296564-3.256258 3.296564zm-2.939929-4.284026v-16.27937h-9.918273v16.27937zm13.894626 45.346016h-55.686796l-.030936-.517247c-.009579-.164357-.016803-.322906-.016803-.482397l-.001256-24.413404c0-1.46807.344219-2.835673 1.59421-3.705965.603011-.42196 1.427126-.740942 2.149482-.740942 8.05522-.020721 16.109812-.028256 24.163776-.030297l24.160007.002669c2.070337 0 3.732386 1.609978 3.744949 3.850385l.015472 4.066295.007279 3.9424v5.801579l-.012701 11.880314z" fill="#9ecc4f" mask="url(#b)"/><path d="m169.941919 1.5891547h-2.858597c.085161.22998007.136003.47266621.136003.72805844v2.08506787c0 1.17912431-.950748 2.12953916-2.122658 2.12953916h-3.917383c-1.170639 0-2.122657-.95422668-2.122657-2.12953916v-2.08506787c0-.25539223.050842-.50061959.13346-.72805844h-53.160491c.082618.22998007.134732.47266621.134732.72805844v2.08506787c0 1.17912431-.950748 2.12953916-2.123929 2.12953916h-3.914841c-1.1731811 0-2.1251993-.95422668-2.1251993-2.12953916v-2.08506787c0-.25539223.0521132-.50061959.1347316-.72805844h-3.4483649v80.0101913h3.4483649c-.0826184-.22998-.1347316-.4726662-.1347316-.729329v-2.0825267c0-1.1816655.9507471-2.1308097 2.1251993-2.1308097h3.914841c1.170639 0 2.123929.952956 2.123929 2.1308097v2.0825267c0 .2579334-.052114.5018902-.134732.729329h53.161762c-.083889-.22998-.134731-.4726662-.134731-.729329v-2.0825267c0-1.1816655.949476-2.1308097 2.122657-2.1308097h3.917383c1.170639 0 2.122658.952956 2.122658 2.1308097v2.0825267c0 .2579334-.050842.5018902-.134732.729329h2.857326zm-63.777591 68.3574446c0 1.1803949-.950748 2.1308097-2.123929 2.1308097h-3.914841c-1.1731811 0-2.1251993-.9542267-2.1251993-2.1308097v-2.0837973c0-1.1816655.9507471-2.1320804 2.1251993-2.1320804h3.914841c1.170639 0 2.123929.9542267 2.123929 2.1320804zm0-10.9246884c0 1.1803949-.950748 2.1308098-2.123929 2.1308098h-3.914841c-1.1731811 0-2.1251993-.9542267-2.1251993-2.1308098v-2.0837973c0-1.1803949.9507471-2.1295391 2.1251993-2.1295391h3.914841c1.170639 0 2.123929.9529561 2.123929 2.1295391zm0-10.9234177c0 1.1816655-.950748 2.1308097-2.123929 2.1308097h-3.914841c-1.1731811 0-2.1251993-.9542266-2.1251993-2.1308097v-2.0837973c0-1.1816655.9507471-2.1308098 2.1251993-2.1308098h3.914841c1.170639 0 2.123929.9542267 2.123929 2.1308098zm0-10.9246884c0 1.1816656-.950748 2.1308098-2.123929 2.1308098h-3.914841c-1.1731811 0-2.1251993-.9529561-2.1251993-2.1308098v-2.0837972c0-1.1816656.9507471-2.1320804 2.1251993-2.1320804h3.914841c1.170639 0 2.123929.9554973 2.123929 2.1320804zm0-10.9234177c0 1.1778537-.950748 2.1308098-2.123929 2.1308098h-3.914841c-1.1731811 0-2.1251993-.9554973-2.1251993-2.1308098v-2.0837973c0-1.1816655.9507471-2.1320803 2.1251993-2.1320803h3.914841c1.170639 0 2.123929.9554972 2.123929 2.1320803zm0-10.9246884c0 1.180395-.950748 2.1308098-2.123929 2.1308098h-3.914841c-1.1731811 0-2.1251993-.9554973-2.1251993-2.1308098v-2.0837972c0-1.1803949.9507471-2.1320804 2.1251993-2.1320804h3.914841c1.170639 0 2.123929.9554973 2.123929 2.1320804zm47.881811 57.3222134c0 1.805534-1.482047 3.2832513-3.292026 3.2832513h-36.880853c-1.809979 0-3.292026-1.4777173-3.292026-3.2832513v-22.9878416c0-1.8055341 1.482047-3.2819807 3.292026-3.2819807h36.880853c1.809979 0 3.292026 1.4764466 3.292026 3.2819807zm.113123-37.3482542c0 1.815699-1.490944 3.3010398-3.30982 3.3010398h-37.071511c-1.818876 0-3.308549-1.4853408-3.308549-3.3010398v-23.1060081c0-1.8144283 1.489673-3.29849859 3.308549-3.29849859h37.071511c1.818876 0 3.30982 1.48407029 3.30982 3.29849859zm13.060063 34.6469414c0 1.1803949-.950748 2.1308097-2.122658 2.1308097h-3.917383c-1.170639 0-2.122657-.9542267-2.122657-2.1308097v-2.0837973c0-1.1816655.949476-2.1320804 2.122657-2.1320804h3.917383c1.170639 0 2.122658.9542267 2.122658 2.1320804zm0-10.9246884c0 1.1803949-.950748 2.1308098-2.122658 2.1308098h-3.917383c-1.170639 0-2.122657-.9542267-2.122657-2.1308098v-2.0837973c0-1.1803949.949476-2.1295391 2.122657-2.1295391h3.917383c1.170639 0 2.122658.9529561 2.122658 2.1295391zm0-10.9234177c0 1.1816655-.950748 2.1308097-2.122658 2.1308097h-3.917383c-1.170639 0-2.122657-.9542266-2.122657-2.1308097v-2.0837973c0-1.1816655.949476-2.1308098 2.122657-2.1308098h3.917383c1.170639 0 2.122658.9542267 2.122658 2.1308098zm0-10.9246884c0 1.1816656-.950748 2.1308098-2.122658 2.1308098h-3.917383c-1.170639 0-2.122657-.9529561-2.122657-2.1308098v-2.0837972c0-1.1816656.949476-2.1320804 2.122657-2.1320804h3.917383c1.170639 0 2.122658.9554973 2.122658 2.1320804zm0-10.9234177c0 1.1778537-.950748 2.1308098-2.122658 2.1308098h-3.917383c-1.170639 0-2.122657-.9554973-2.122657-2.1308098v-2.0837973c0-1.1816655.949476-2.1320803 2.122657-2.1320803h3.917383c1.170639 0 2.122658.9554972 2.122658 2.1320803zm0-10.9246884c0 1.180395-.950748 2.1308098-2.122658 2.1308098h-3.917383c-1.170639 0-2.122657-.9554973-2.122657-2.1308098v-2.0837972c0-1.1803949.949476-2.1320804 2.122657-2.1320804h3.917383c1.170639 0 2.122658.9554973 2.122658 2.1320804z" fill="#f1644b" mask="url(#b)"/><g fill="#aa99c9" fill-rule="nonzero"><path d="m190.008055 118.856762.06738-.002313.202632-.017721.283463-.033433.551385-.075029 1.413532-.213695 4.114125-.676v-6.417866l-13.686253 1.919415.604476 6.757576.622534 6.83318.636344 6.874152.645903 6.880491.651215 6.852199.977487 10.14949c.108211 1.115526.216201 2.226266.323881 3.331499 2.244254-.219873 4.534679-.451595 6.861181-.69245l4.698114-.493084c.790039-.083943 1.583338-.168699 2.379525-.254168l4.808769-.520571 7.303803-.802397 12.265177-1.354553 4.885987-.533775 4.847589-.521939c5.626144-.600147 11.137253-1.166276 16.405082-1.663902-.057466-.377108-.110561-.8521-.159691-1.38768l-.070771-.844148-.065129-.907059-.087779-1.417357-.20156-3.779696c-.005977-.105888-.011809-.20624-.0175-.300471-.377122.045061-.863464.067592-1.404401.078857l-.845524.009857-1.772851.002817-.845389.009857c-.540816.011265-1.026978.033796-1.403858.078857v2.759279c-6.421305.606888-12.851014 1.281323-19.282875 1.975881l-12.864738 1.401893c-6.431951.70031-12.861886 1.389126-19.283552 2.019024l-2.678814-26.182008zm9.708286 24.890082h62.646972v-48.3391642h-62.646972zm5.78815-42.762121h51.250918v19.671611l-6.451306-10.9748-8.682708 20.570918-7.416158-8.816655-20.319735 16.672644h-8.381011zm19.716341 12.355397c0 3.477383-2.774072 6.236662-6.331784 6.236662-3.436263 0-6.390581-2.759279-6.390581-6.236662 0-3.418898 2.954318-6.23762 6.390581-6.23762 3.478652 0 6.208105 2.694837 6.327696 6.010607z" mask="url(#b)"/><path d="m8.50178283 37.8554944.06738008-.0023133.20263199-.0177206.28346326-.0334334.76303076-.1056252 1.46084558-.2239078 3.8551654-.6351912v-6.4178655l-13.68625347 1.9194154.29971525 3.3671728.61403634 6.799707.62997002 6.8579949.6416547 6.8816506.64909042 6.8706739.65227715 6.8250649.97509621 10.076322c1.49616923-.146582 3.0128591-.29843 4.54707848-.4547395l4.6522556-.4815092 4.739486-.5034529 4.8087696-.520571 19.5689796-2.1569499 4.8859864-.5337752 4.8475896-.5219388c5.6261437-.6001474 11.1372525-1.1662761 16.4050819-1.6639024-.0574661-.3771079-.1105608-.8520995-.1596905-1.3876792l-.0707716-.8441486-.0651287-.9070589-.0597907-.9419975-.0547573-.9489644-.1747909-3.3060911c-.0059777-.1058884-.0118091-.2062396-.0175006-.300471-.4714018.0563265-1.1134607.077449-1.8194867.0853699l-.8711072.0045765-1.3321818.0015842-.8453886.0098572c-.5408168.0112653-1.0269781.0337959-1.4038585.0788571v2.7592795c-6.421305.6068881-12.8510139 1.281323-19.2828747 1.9758805l-12.864738 1.4018932c-6.4319511.70031-12.861886 1.3891261-19.2835524 2.0190242l-2.86056387-27.9915709zm9.70828547 24.8900819h62.6469723v-48.3391642h-62.6469723zm5.7881507-42.7621214h51.250918v19.6716113l-6.4513061-10.9747995-8.682708 20.5709179-7.4161585-8.8166555-20.3197345 16.6726445h-8.3810109zm19.7163403 12.3553979c0 3.4773825-2.7740713 6.236662-6.3317839 6.236662-3.4362627 0-6.3905811-2.7592795-6.3905811-6.236662 0-3.4188988 2.9543184-6.2376208 6.3905811-6.2376208 3.4786523 0 6.2081048 2.6948374 6.327696 6.0106077z" mask="url(#b)"/></g></g></g></g></svg>
`,ot=m`
  <svg viewBox="0 0 787 400" xmlns="http://www.w3.org/2000/svg"><g fill="#999" fill-rule="evenodd"><path d="m392.387152 0c90.212937 0 163.325255 73.1823935 163.325255 163.414428 0 34.161159-10.533846 65.858529-28.422252 92.099564l96.093209 96.172128c5.880272 5.034454 9.616636 12.49332 9.616636 20.857826 0 15.159612-12.300741 27.456054-27.47128 27.456054-7.270154 0-13.876316-2.846294-18.788734-7.464492l-97.84885-97.516523c-27.037997 19.929684-60.361413 31.804247-96.503984 31.804247-90.274834 0-163.387152-73.126143-163.387152-163.408804 0-90.2320345 73.112318-163.414428 163.387152-163.414428zm.132028 36c-70.323133 0-127.51918 57.1785661-127.51918 127.511499 0 70.294604 57.196047 127.488501 127.51918 127.488501 70.246413 0 127.48082-57.193897 127.48082-127.488501 0-70.3329329-57.234407-127.511499-127.48082-127.511499z"/><path d="m378.080616 218.418605v24.781395h24.697248v-24.781395zm-36.267131-84.83721h18.912307c0-5.209302.593328-10.046511 1.779982-14.511628 1.186655-4.465116 3.077886-8.334883 5.673692-11.609302 2.595807-3.274418 5.822025-5.87907 9.678652-7.8139534 3.856627-1.9348837 8.454914-2.9023256 13.79486-2.9023256 8.009918 0 14.351104 2.3069768 19.023556 6.92093 4.672453 4.613954 7.305342 11.013954 7.89867 19.2.296663 5.506977-.37083 10.195349-2.00248 14.065117-1.63165 3.869767-3.819544 7.404651-6.563683 10.604651s-5.710775 6.251163-8.89991 9.153488c-3.189134 2.902326-6.229936 6.065116-9.122407 9.488372-2.89247 3.423256-5.339945 7.330233-7.342425 11.72093-2.00248 4.390698-3.152051 9.711628-3.448715 15.962791v10.493023h18.912308v-8.706976c0-3.869768.556244-7.330233 1.668733-10.381396 1.112488-3.051163 2.595807-5.879069 4.449954-8.483721 1.854148-2.604651 3.930794-5.060465 6.229937-7.367442 2.299143-2.306976 4.635369-4.576744 7.008679-6.809302 2.373309-2.381395 4.709535-4.837209 7.008678-7.367442 2.299144-2.530232 4.338706-5.283721 6.118688-8.260465s3.226217-6.288372 4.338706-9.934884c1.112489-3.646511 1.668733-7.776744 1.668733-12.390697 0-7.144186-1.149572-13.469768-3.448715-18.976744-2.299143-5.506977-5.52536-10.1581399-9.678651-13.9534888-4.153292-3.7953488-9.085325-6.6976744-14.7961-8.7069767s-12.051961-3.0139535-19.023556-3.0139535c-7.713255 0-14.684851 1.3395349-20.914788 4.0186047-6.229936 2.6790697-11.495716 6.4372093-15.797339 11.2744186-4.301623 4.8372097-7.602006 10.5302327-9.901149 17.0790697s-3.374549 13.618605-3.226217 21.209302z" fill-rule="nonzero"/></g></svg>
`,P,st=P=class extends c{constructor(){super(...arguments),this.placeholderType=null,this.detailMessage=``}static get MESSAGE_EMPTY_QUERY(){return o(`To begin searching, enter a search term in the box above and hit "Go".`)}static get MESSAGE_NO_SEARCH_RESULTS(){return o(`Your search did not match any items in the Archive. Try different keywords or a more general search.`)}static get MESSAGE_NO_COLLECTION_RESULTS(){return o(`Your search did not match any items in this collection. Try different keywords or a more general search.`)}static get MESSAGE_NO_VIEWABLE_MEMBERS(){return o(`This collection contains no viewable items.`)}static get MESSAGE_QUERY_ERROR(){return o(a`The search engine encountered an error, which might be related to your
      search query.
      <a
        href="https://help.archive.org/help/search-building-powerful-complex-queries/"
      >
        Tips for constructing search queries.
      </a> `)}static get MESSAGE_COLLECTION_ERROR(){return o(a`The search engine encountered an error while loading this collection.
      If the problem persists, please let us know at
      <a href="mailto:info@archive.org">info@archive.org</a>.`)}static get QUERY_ERROR_DETAILS_MESSAGE(){return o(`Error details:`)}render(){return this.placeholderType?a`${this.placeholderTemplate}`:n}get placeholderTemplate(){return a`
      <div
        class="placeholder ${this.placeholderType} ${this.isMobileView?`mobile`:`desktop`}"
      >
        ${te(this.placeholderType,[[`empty-query`,()=>this.emptyQueryTemplate],[`empty-collection`,()=>this.emptyCollectionTemplate],[`no-results`,()=>this.noResultsTemplate],[`query-error`,()=>this.queryErrorTemplate],[`collection-error`,()=>this.collectionErrorTemplate]])}
      </div>
    `}get emptyQueryTemplate(){return a`
      <h2 class="title" data-testid="empty-query-text-msg">
        ${P.MESSAGE_EMPTY_QUERY}
      </h2>
      <div>${at}</div>
    `}get emptyCollectionTemplate(){return a`
      <h2 class="title" data-testid="empty-collection-text-msg">
        ${P.MESSAGE_NO_VIEWABLE_MEMBERS}
      </h2>
      <div>${ot}</div>
    `}get noResultsTemplate(){return a`
      <h2 class="title" data-testid="empty-results-text-msg">
        ${this.isCollection?P.MESSAGE_NO_COLLECTION_RESULTS:P.MESSAGE_NO_SEARCH_RESULTS}
      </h2>
      <div>${ot}</div>
    `}get queryErrorTemplate(){return a`
      <h2 class="title" data-testid="error-query-text-msg">
        ${P.MESSAGE_QUERY_ERROR}
      </h2>
      <div>${ot}</div>
      <p class="error-details">
        ${P.QUERY_ERROR_DETAILS_MESSAGE} ${this.detailMessage}
      </p>
    `}get collectionErrorTemplate(){return a`
      <h2 class="title" data-testid="error-collection-text-msg">
        ${P.MESSAGE_COLLECTION_ERROR}
      </h2>
      <div>${ot}</div>
      <p class="error-details">
        ${P.QUERY_ERROR_DETAILS_MESSAGE} ${this.detailMessage}
      </p>
    `}static get styles(){return e`
      :host {
        text-align: center;
        width: 100%;
      }

      a {
        text-decoration: none;
      }
      a:link {
        color: var(--ia-theme-link-color, #4b64ff);
      }
      a:hover {
        text-decoration: underline;
      }

      .placeholder {
        display: block;
      }

      .desktop svg {
        max-height: 40rem;
      }
      .desktop .title,
      .desktop .error-details {
        margin: 4rem 0;
      }

      .mobile svg {
        max-height: 20rem;
      }
      .mobile .title,
      .mobile .error-details {
        margin: 2rem 0.5;
      }

      .error-details {
        font-size: 1.2rem;
        word-break: break-word;
      }
    `}};C([t({type:String})],st.prototype,`placeholderType`,void 0),C([t({type:Boolean})],st.prototype,`isMobileView`,void 0),C([t({type:Boolean})],st.prototype,`isCollection`,void 0),C([t({type:String})],st.prototype,`detailMessage`,void 0),st=P=C([x(`empty-placeholder`),i()],st);var ct=m`
  <svg viewBox="0 0 300 300"
    xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m280 262.122891v-156.834044c0-4.877026-2.063112-9.1444235-6.189337-12.8021929s-8.220563-5.4866541-12.283013-5.4866541h-225.4946834c-4.4642309 0-8.2524498 1.726748-11.3646565 5.1802441s-4.6683101 7.8230299-4.6683101 13.1086029v156.834044c0 5.279189 1.5210273 9.578505 4.5630818 12.897946 3.0420545 3.319442 6.8653495 4.979163 11.4698848 4.979163h225.4946834c4.06245 0 8.156788-1.726748 12.283013-5.180244s6.189337-7.685784 6.189337-12.696865zm-200.9382244-131.440315v7.918783c0 1.487366-.7780516 3.00984-2.334155 4.567424-1.5561034 1.557585-3.1472828 2.336377-4.7735384 2.336377h-14.8180581c-1.7601825 0-3.4183254-.743683-4.9744288-2.231048-1.5561033-1.487365-2.334155-3.044949-2.334155-4.672753v-7.918783c0-1.761858.8131278-3.964179 2.4393834-6.606966 1.6262555-2.642786 3.2493223-3.964179 4.8692004-3.964179h14.8180581c1.4859512 0 3.0420545 1.321393 4.6683101 3.964179 1.6262556 2.642787 2.4393833 4.845108 2.4393833 6.606966zm169.2740724 0v7.918783c0 1.627804-.711089 3.185388-2.133265 4.672753-1.422177 1.487365-3.080319 2.231048-4.974429 2.231048h-131.114463c-2.028037 0-3.753143-.743683-5.175319-2.231048-1.422177-1.487365-2.133265-3.044949-2.133265-4.672753v-7.918783c0-1.895912.742975-4.130152 2.228927-6.702719 1.485951-2.572567 3.175981-3.858851 5.070091-3.858851h131.114463c1.760182 0 3.383249 1.286284 4.8692 3.858851 1.485952 2.572567 2.228927 4.806807 2.228927 6.702719zm-169.2740724 50.988539v7.918784c0 1.487365-.7780516 2.977922-2.334155 4.471671s-3.1472828 2.237431-4.7735384 2.231088h-14.8180581c-1.7601825 0-3.4183254-.743723-4.9744288-2.231088-1.5561033-1.487365-2.334155-2.977922-2.334155-4.471671v-7.918784c0-1.895912.8131278-4.165261 2.4393834-6.808047 1.6262555-2.642786 3.2493223-3.964179 4.8692004-3.964179h14.8180581c1.4859512 0 3.0420545 1.353311 4.6683101 4.059932 1.6262556 2.706622 2.4393833 4.940861 2.4393833 6.702719zm169.2740724 0v7.918784c0 1.487365-.711089 2.977922-2.133265 4.471671-1.422177 1.493749-3.080319 2.237431-4.974429 2.231088h-131.114463c-2.028037 0-3.753143-.743723-5.175319-2.231088-1.422177-1.487365-2.133265-2.977922-2.133265-4.471671v-7.918784c0-2.029966.742975-4.331233 2.228927-6.9038 1.485951-2.572567 3.175981-3.858851 5.070091-3.858851h131.114463c1.760182 0 3.383249 1.321393 4.8692 3.964179 1.485952 2.642787 2.228927 4.912136 2.228927 6.808048zm-169.2740724 51.400278v6.9038c0 1.761858-.7780516 3.421579-2.334155 4.979163s-3.1472828 2.336376-4.7735384 2.336376h-14.8180581c-1.7601825 0-3.4183254-.778792-4.9744288-2.336376-1.5561033-1.557584-2.334155-3.217305-2.334155-4.979163v-6.9038c0-2.029966.7780517-4.366342 2.334155-7.009129 1.5561034-2.642786 3.2142463-3.964179 4.9744288-3.964179h14.8180581c1.4859512 0 3.0420545 1.321393 4.6683101 3.964179 1.6262556 2.642787 2.4393833 4.979163 2.4393833 7.009129zm169.2740724 0v6.9038c0 1.761858-.711089 3.421579-2.133265 4.979163-1.422177 1.557584-3.080319 2.336376-4.974429 2.336376h-131.114463c-2.028037 0-3.753143-.778792-5.175319-2.336376-1.422177-1.557584-2.133265-3.217305-2.133265-4.979163v-6.9038c0-2.170404.742975-4.54189 2.228927-7.114457 1.485951-2.572567 3.175981-3.858851 5.070091-3.858851h131.114463c1.760182 0 3.383249 1.321393 4.8692 3.964179 1.485952 2.642787 2.228927 4.979163 2.228927 7.009129z"
      fill="black"
    />
  </svg>
`,lt=[(e,t)=>o(t?r`${e} byte`:r`${e} bytes`),(e,t)=>o(t?r`${e} kilobyte`:r`${e} kilobytes`),(e,t)=>o(t?r`${e} megabyte`:r`${e} megabytes`),(e,t)=>o(t?r`${e} gigabyte`:r`${e} gigabytes`),(e,t)=>o(t?r`${e} terabyte`:r`${e} terabytes`),(e,t)=>o(t?r`${e} petabyte`:r`${e} petabytes`),(e,t)=>o(t?r`${e} exabyte`:r`${e} exabytes`),(e,t)=>o(t?r`${e} zettabyte`:r`${e} zettabytes`),(e,t)=>o(t?r`${e} yottabyte`:r`${e} yottabytes`)];function ut(e,t,r=M()){let i=e;if(i===void 0)return n;let a=0;for(;i>1024&&a<lt.length-1;)i/=1024,a+=1;let o=10**t;return i=Math.round(i*o)/o,lt[a](new Intl.NumberFormat(r).format(i),i===1)}var dt=e`
  /* Include .sr-only styles for all tiles */
  ${j}

  .container {
    background-color: ${e`var(--tileBackgroundColor, #ffffff)`};
    border: 1px #2c2c2c;
    border-radius: ${e`var(--tileCornerRadius, 4px)`};
    box-shadow: var(--tileBoxShadow, 1px 1px 2px 0);
    box-sizing: border-box;
    height: 100%;
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  image-block {
    display: block;
    position: relative;
    text-align: center;
  }

  .tile-details {
    display: flex;
    flex-direction: column;
    height: 100%;
    row-gap: 10px;
    font-family: 'Helvetica Neue', ui-sans-serif, system-ui, sans-serif;
  }

  .item-info {
    display: flex;
    flex-direction: column;
    row-gap: 5px;
    flex-grow: 1;
  }

  #title {
    padding: 0 5px;
  }

  .created-by,
  .date-sorted-by,
  .volume-issue,
  .archivist-since {
    display: flex;
    justify-content: left;
    align-items: flex-start;
    padding: 0 5px;
  }

  .truncated {
    flex: 1;
    color: #2c2c2c;
    min-width: 0; /* Important for long words! */
    text-align: left;
    line-height: 15px;
    text-overflow: ellipsis;
    overflow: hidden;
    word-wrap: break-word;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
  }

  h4.truncated,
  h3.truncated {
    display: -webkit-box;
    margin: 0px;
    line-height: 15px;
    font-size: 14px;
    font-weight: 500;
    padding-bottom: 1px;
  }

  span {
    display: -webkit-box;
    font-size: 1.4rem;
    line-height: 15px;
    overflow: hidden;
    word-wrap: break-word;
    -webkit-line-clamp: 1;
    -webkit-box-orient: vertical;
    padding-bottom: 1px;
  }

  .container:hover > .tile-details > .item-info > #title > .truncated {
    text-decoration: underline;
  }

  /** this is a workaround for Safari 15 where the hover effects are not working */
  #title:hover > .truncated {
    text-decoration: underline;
  }

  .info-button {
    position: absolute;
    right: 10px;
    top: 10px;
    margin: 0;
    padding: 0;
    border: none;
    border-radius: 50%;
    display: flex;
    justify-content: center;
    align-items: center;
    background: rgba(220, 220, 220, 0.5);
    color: white;
    font-size: 2.4rem;
    font-weight: bold;
    line-height: 1;
    text-shadow: black 1px 1px 3px;
    overflow: visible;
    aspect-ratio: 1 / 1;
    z-index: 1;
  }

  .hidden {
    display: none;
  }
`;function ft(e,t=`short`,{locale:n=`en-US`,useLocalTime:r=!1}={}){if(!e)return``;let i=r?{}:{timeZone:`UTC`};switch(t){case`year-only`:return`${e.getUTCFullYear()}`;case`short`:i.month=`short`,i.year=`numeric`;break;case`long`:i.year=`numeric`,i.month=`short`,i.day=`2-digit`;break;default:break}return new Intl.DateTimeFormat(n,i).format(e)}var pt=class{constructor(e={}){this.model=e.model,this.baseNavigationUrl=e.baseNavigationUrl,this.collectionPagePath=e.collectionPagePath??`/details/`,this.sortParam=e.sortParam,this.creatorFilter=e.creatorFilter}get firstCreatorMatchingFilter(){let e;if(this.creatorFilter&&this.model?.creators.length){let t=this.creatorFilter;e=this.model.creators.find(e=>e.normalize(`NFD`).replace(/[^A-Z]+/gi,``).toUpperCase().startsWith(t))}return e??this.model?.creator}get accountLabel(){return this.model?.dateAdded?o(r`Archivist since ${this.model.dateAdded.getFullYear()}`):``}get dateLabel(){switch(this.sortParam?.field){case`publicdate`:return o(`Archived`);case`reviewdate`:return o(`Reviewed`);case`addeddate`:return o(`Added`);case`date`:return o(`Published`);default:return``}}get viewsLabel(){return this.sortParam?.field===`week`?o(`Weekly views`):o(`All-time views`)}itemPageUrl(e,t=!1){if(!e||this.baseNavigationUrl==null)return n;let r=t?this.collectionPagePath:`/details/`;return`${this.baseNavigationUrl}${r}${e}`}webArchivesCaptureLink(e,t){return a` <a href=${`https://web.archive.org/web/${t.toISOString().replace(/[TZ:-]/g,``).replace(/\..*/,``)}/${e}`}> ${ft(t,`long`)} </a> `}},F=class extends c{constructor(){super(...arguments),this.tileActions=[],this.sortParam=null,this.defaultSortParam=null,this.loggedIn=!1,this.suppressBlurring=!1,this.useLocalTime=!1,this.displayValueProvider=new pt}willUpdate(e){(e.has(`model`)||e.has(`baseNavigationUrl`)||e.has(`collectionPagePath`)||e.has(`sortParam`)||e.has(`defaultSortParam`)||e.has(`creatorFilter`))&&(this.displayValueProvider=new pt({model:this.model,baseNavigationUrl:this.baseNavigationUrl,collectionPagePath:this.collectionPagePath,sortParam:this.sortParam??this.defaultSortParam??void 0,creatorFilter:this.creatorFilter}))}getFormattedDate(e,t){let{useLocalTime:n}=this;return ft(e,t,{useLocalTime:n})}renderTileActions(e=``){return this.tileActions.length?a`
      <div class=${g({"tile-actions":!0,...e&&{[e]:!0}})}>
        ${this.tileActions.map(e=>a`
            <button
              class="tile-action-btn"
              @click=${t=>this.handleTileActionClick(t,e)}
            >
              ${e.label}
            </button>
          `)}
      </div>
    `:n}handleTileActionClick(e,t){e.preventDefault(),e.stopPropagation(),this.dispatchEvent(new PointerEvent(`pointerdown`,{bubbles:!0,composed:!0})),this.dispatchEvent(new CustomEvent(`tileActionClicked`,{detail:{actionId:t.id,model:this.model},bubbles:!0,composed:!0}))}};C([t({type:Object})],F.prototype,`model`,void 0),C([t({type:Array})],F.prototype,`tileActions`,void 0),C([t({type:Number})],F.prototype,`currentWidth`,void 0),C([t({type:Number})],F.prototype,`currentHeight`,void 0),C([t({type:String})],F.prototype,`baseNavigationUrl`,void 0),C([t({type:String})],F.prototype,`baseImageUrl`,void 0),C([t({type:String})],F.prototype,`collectionPagePath`,void 0),C([t({type:Object})],F.prototype,`sortParam`,void 0),C([t({type:Object})],F.prototype,`defaultSortParam`,void 0),C([t({type:String})],F.prototype,`creatorFilter`,void 0),C([t({type:Number})],F.prototype,`mobileBreakpoint`,void 0),C([t({type:Boolean})],F.prototype,`loggedIn`,void 0),C([t({type:Boolean})],F.prototype,`suppressBlurring`,void 0),C([t({type:Boolean})],F.prototype,`useLocalTime`,void 0);var mt=class extends c{render(){return a`<div class="icon-overlay">${this.iconTemplate}</div>`}get iconTemplate(){return this.type?a`${he[this.type]??n}`:n}static get styles(){return e`
      :host {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        width: 100%;
        z-index: 2;
        background-color: rgb(255, 254, 203);
        display: flex;
        justify-content: center;
        border: 1px solid #2c2c2c;
      }

      .icon-overlay {
        height: 20px;
        width: 20px;
        padding: 2px;
        padding: 2px;
      }

      :host(.list-detail) .icon-overlay {
        height: 20px;
        width: 20px;
      }
      :host(.list-compact) .icon-overlay {
        height: 15px;
        width: 15px;
      }
    `}};C([t({type:String})],mt.prototype,`type`,void 0),mt=C([x(`icon-overlay`)],mt);var ht=class extends c{render(){return a`
      <div class="overlay no-preview">
        <div class="icon-overlay">${this.iconTemplate}</div>
        <p class="text-overlay">${this.textTemplate}</p>
      </div>
    `}get iconTemplate(){return this.type?a`${he[this.type]??n}`:n}get textTemplate(){return this.type?a`${me[this.type]??n}`:n}static get styles(){return e`
      :host {
        align-items: center;
        display: flex;
      }

      .overlay {
        border: 1px solid #2c2c2c;
        border-radius: 1px;
        position: absolute;
        right: 0;
        left: 0;
        top: 50%;
        transform: translate(0px, -50%);
        width: auto;
        height: auto;
        padding: 5px;
        background-color: #fffecb;
        display: flex;
        align-items: center;
        border-radius: 1px;
      }

      .icon-overlay {
        width: 40px;
        height: 40px;
        display: inherit;
      }
      .text-overlay {
        margin: 0;
        vertical-align: bottom;
        padding-left: 5px;
        text-align: center;
        color: #2c2c2c;
        font-size: 1.4rem;
        line-height: 2rem;
      }
    `}};C([t({type:String})],ht.prototype,`type`,void 0),ht=C([x(`text-overlay`),i()],ht);var I=class extends c{constructor(){super(...arguments),this.isCompactTile=!1,this.isListTile=!1,this.loggedIn=!1,this.suppressBlurring=!1,this.viewSize=`desktop`}render(){return this.model?.identifier?a`
      <div class=${g(this.baseClass)}>
        <item-image
          .model=${this.model}
          .baseImageUrl=${this.baseImageUrl}
          .isListTile=${this.isListTile}
          .isCompactTile=${this.isCompactTile}
          .loggedIn=${this.loggedIn}
          .suppressBlurring=${this.suppressBlurring}
          style="--imgHeight: 100%; --imgWidth: 100%"
        >
        </item-image>
        ${this.textOverlayTemplate} ${this.iconOverlayTemplate}
      </div>
    `:n}get baseClass(){return{container:!0,list:this.isListTile&&!this.isCompactTile,"list-compact":this.isListTile&&this.isCompactTile,collection:this.model?.mediatype===`collection`,[this.viewSize]:!0,"search-image":this.model?.mediatype===`search`}}get iconOverlayTemplate(){if(!this.isListTile)return n;let{overlayType:e}=this;return e?a`
      <icon-overlay
        class=${this.isCompactTile?`list-compact`:`list-detail`}
        .type=${this.overlayType}
      >
      </icon-overlay>
    `:n}get textOverlayTemplate(){if(this.isListTile)return n;let{overlayType:e}=this;return e?a` <text-overlay .type=${this.overlayType}></text-overlay> `:n}get overlayType(){if(!this.suppressBlurring){if(this.model?.loginRequired&&!this.loggedIn)return`login-required`;if(this.model?.contentWarning)return`content-warning`}}static get styles(){return e`
      div {
        display: flex;
        justify-content: center;
        position: relative;
        background-color: ${e`var(--imageBlockBackgroundColor, #f1f1f4)`};
        border-radius: 4px;
      }

      .grid {
        height: var(--imgBlockHeight, 16rem);
        flex: 1;
        position: initial;
        padding: 5px;
        border-radius: 4px 4px 0 0;
      }

      .collection.grid,
      .grid.search-image {
        display: block;
      }

      /** tile-list view */
      .list {
        border-radius: 0;
        background-color: var(--imageBlockListBackgroundColor, #ebebee);
        box-shadow: 1px 1px 2px rgb(0, 0, 0, 0.2);
      }

      .list.desktop {
        width: 100px;
        max-width: 100%;
        height: 100px;
        max-width: 100%;
        display: inline-block;
        position: relative;
        text-align: center;
      }

      .list.mobile {
        width: var(--image-width, 90px);
        height: 90px;
      }

      /** tile-list-compact view */
      .container.list-compact {
        display: block;
        text-align: center;
      }

      .list-compact.desktop {
        width: 45px;
        height: 45px;
      }

      .list-compact.mobile {
        width: 30px;
        height: 30px;
      }
    `}};C([t({type:String})],I.prototype,`baseImageUrl`,void 0),C([t({type:Boolean})],I.prototype,`isCompactTile`,void 0),C([t({type:Boolean})],I.prototype,`isListTile`,void 0),C([t({type:Boolean})],I.prototype,`loggedIn`,void 0),C([t({type:Boolean})],I.prototype,`suppressBlurring`,void 0),C([t({type:Object})],I.prototype,`model`,void 0),C([t({type:String})],I.prototype,`viewSize`,void 0),I=C([x(`image-block`)],I);var gt=class extends F{constructor(){super(...arguments),this.showInfoButton=!1,this.layoutType=`default`}render(){return a`
      <div class=${g({container:!0,minimal:this.layoutType===`minimal`})}>
        ${this.infoButtonTemplate}
        <div class="tile-details">
          <div class="item-info">
            ${this.getImageBlockTemplate} ${this.getTitleTemplate}
          </div>
        </div>

        ${this.getTileStatsTemplate}
      </div>
    `}get getImageBlockTemplate(){return a`
      <image-block
        .model=${this.model}
        .baseImageUrl=${this.baseImageUrl}
        .viewSize=${`grid`}
        .suppressBlurring=${this.suppressBlurring}
      >
      </image-block>
    `}get getTitleTemplate(){return a`<div id="title">
      <h3 class="truncated">${this.model?.title}</h3>
    </div>`}get getTileStatsTemplate(){return a`
      <div id="item-stats">
        <div id="item-mediatype">${ct}</div>

        <div id="stats-row">
          ${this.getItemsTemplate} ${this.getSizeTemplate}
        </div>
      </div>
    `}get getItemsTemplate(){let e=this.model?.itemCount,t=e===void 0?void 0:new Intl.NumberFormat(M()).format(e);return a`<span id="item-count">${o(e===1?r`${t} item`:r`${t} items`)}</span>`}get getSizeTemplate(){let e=this.model?.collectionSize??0;return e?a`<span id="item-size">${ut(e,1)}</span>`:``}get infoButtonTemplate(){return this.showInfoButton?a`<button class="info-button" @click=${this.infoButtonPressed}>
          &#9432;
          <span class="sr-only">${o(`More info`)}</span>
        </button>`:n}infoButtonPressed(e){e.preventDefault();let t=new CustomEvent(`infoButtonPressed`,{detail:{x:e.clientX,y:e.clientY}});this.dispatchEvent(t)}static get styles(){let t=e`var(--tileBorderColor, #555555)`,n=e`var(--tileBackgroundColor, #666666)`,r=e`#fff`;return[dt,e`
        .container {
          background-color: ${n};
          border: 1px solid ${t};
        }

        .item-info {
          flex-grow: initial;
        }

        h4.truncated,
        h3.truncated {
          color: ${r};
        }

        #item-mediatype svg {
          filter: invert(100%);
          height: 2.5rem;
          align-items: baseline;
        }

        .container:hover > #title {
          text-decoration: underline;
        }

        /* this is a workaround for Safari 15 where the hover effects are not working */
        image-block:hover > #title {
          text-decoration: underline;
        }

        #item-stats {
          display: flex;
          padding: 0 5px 5px;
          align-items: center;
        }

        #stats-row {
          display: flex;
          align-items: baseline;
          color: ${r};
          flex-direction: column;
          margin-left: 10px;
        }

        .minimal #item-stats {
          display: none;
        }

        .minimal .truncated {
          -webkit-line-clamp: initial;
        }

        .minimal .item-info {
          padding-bottom: 5px;
        }
      `]}};C([t({type:Boolean})],gt.prototype,`showInfoButton`,void 0),C([t({type:String})],gt.prototype,`layoutType`,void 0),gt=C([x(`collection-tile`),i()],gt);function _t(e){return e?e.toISOString().endsWith(`-01-01T00:00:00.000Z`):!1}var vt=a`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      d="m81.0388846 100-30.9636029-22.5595033-30.7410319 22.5595033 10.6670595-37.3922042-30.0013093-25.2155916h37.5556428l12.5196389-37.3922042 12.3690754 37.3922042h37.5556429l-29.7034563 25.2155916z"
      fill="#333"
    />
  </svg>
`,yt=a`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      d="m69.0481858 61.4006179 13.2757509 38.38311-32.1794134-24.4730516-32.2508245 24.6893237 13.2757507-38.4552008-31.1694495-21.9945074 38.1677832-.5115002 11.8339172-39.0387916 11.6162817 38.8946104 38.3820179.6556814zm-2.8122554 15.58874-6.7092871-19.2276004 18.0365221-11.7267421-21.6445066.5115-5.9169584-19.5914862-5.9169587 19.5914862-21.6445064-.5115 18.036522 11.7267421-6.5664638 19.3752146 16.0880061-12.3824233z"
    />
  </svg>
`,bt=class extends c{constructor(){super(...arguments),this.title=``,this.body=``,this.starRating=0,this.viewSize=`desktop`}render(){return!this.title&&!this.body&&!this.starRating?n:a`
      <div class="review-container">
        <div class="snippet-view ${this.viewSize}">
          ${this.starsTemplate}
          <p class="review-title">${this.title}</p>
          <p class="review-body">${this.body}</p>
        </div>
      </div>
    `}get starsTemplate(){if(this.starRating<=0)return n;let e=Math.min(5,this.starRating),t=Math.min(5,5-this.starRating);return a`
      <div class="star-rating">
        <span class="sr-only">${this.starRating} ${o(`out of 5 stars`)}</span>
        ${Array(e).fill(this.filledStarTemplate)}
        ${Array(t).fill(this.unfilledStarTemplate)}
      </div>
    `}get filledStarTemplate(){return a`<span aria-hidden="true">${vt}</span>`}get unfilledStarTemplate(){return a`
      <span class="unfilled-star" aria-hidden="true">
        ${yt}
      </span>
    `}static get styles(){return[j,e`
        .review-container {
          display: flex;
          flex-direction: row;
          flex-wrap: wrap;
          width: calc(100% - 10px);
          border: 1px solid #ccc;
          margin-top: var(--containerTopMargin, 10px);
          margin-left: var(--containerLeftMargin, 0px);
          padding: 5px;
          box-sizing: border-box;
        }

        .review-title,
        .review-body {
          display: -webkit-box;
          font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
          overflow: hidden;
          overflow-wrap: break-word;
          word-break: break-word;
          -webkit-line-clamp: var(--maxLines, 3);
          -webkit-box-orient: vertical;
          margin: 0;
        }

        .review-title {
          font-size: 1.4rem;
          line-height: 2rem;
          max-height: 6rem;
        }

        .review-title > a[href] {
          color: inherit;
          text-decoration: none;
        }

        .review-title > a[href]:hover {
          text-decoration: underline;
        }

        .review-body {
          font-size: 1rem;
          line-height: 1.4rem;
          max-height: 4.2rem;
        }

        .star-rating svg {
          width: 10px;
          height: 10px;
        }

        .unfilled-star {
          fill: #ccc;
        }

        .list {
          margin: 0;
          padding-left: 5px;
        }
      `]}};C([t({type:String})],bt.prototype,`title`,void 0),C([t({type:String})],bt.prototype,`body`,void 0),C([t({type:Number})],bt.prototype,`starRating`,void 0),C([t({type:String})],bt.prototype,`viewSize`,void 0),bt=C([x(`review-block`),i()],bt);var xt=class extends c{constructor(){super(...arguments),this.snippets=[],this.viewSize=`desktop`}render(){return this.snippets?.length?a`
      <div class="container">
        <div class="snippet-view ${this.viewSize}">
          ${this.ellipsisJoinedSnippets}
        </div>
      </div>
    `:a`${n}`}get ellipsisJoinedSnippets(){return a`
      &hellip; ${u(this.snippetTemplates,a` &hellip; `)} &hellip;
    `}get snippetTemplates(){return this.snippets?.map(e=>{let t=e.matchAll(/{{{(.+?)}}}/gs),n=[],r=0;for(let i of t)i.index!=null&&(n.push(a`
            ${e.slice(r,i.index)}
            <mark>${i[1]}</mark>
          `),r=i.index+i[0].length);return n.push(a`${e.slice(r)}`),a`<span>${n}</span>`})}static get styles(){return e`
      .container {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        width: calc(100% - 10px);
        border-left: 5px solid #194880;
        margin-top: var(--containerTopMargin, 10px);
        margin-left: var(--containerLeftMargin, 0px);
        border-radius: 3px;
        box-sizing: border-box;
      }

      .snippet-view {
        display: -webkit-box;
        font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
        font-size: 14px;
        overflow: hidden;
        overflow-wrap: break-word;
        -webkit-line-clamp: var(--maxLines, 3);
        -webkit-box-orient: vertical;
        margin-left: 5px;
      }

      .grid {
        font-size: 1.2rem;
        line-height: 1.5rem;
      }

      .list {
        margin: 0;
        padding-left: 15px;
        font-size: 1.4rem;
        line-height: 2rem;

        /*
         * Safari doesn't always respect the line-clamping rules,
         * so we add this to ensure these fields still get truncated.
         */
        max-height: 6rem;
      }

      mark {
        /* blue, 20% transparency */
        background-color: #0000ff33;
        color: inherit;
      }
    `}};C([t({type:Array})],xt.prototype,`snippets`,void 0),C([t({type:String})],xt.prototype,`viewSize`,void 0),xt=C([x(`text-snippet-block`)],xt);var St=e`
  .drop-shadow {
    overflow: hidden;
    height: 100%;
  }

  .list-box {
    overflow: hidden;
    box-sizing: border-box;
    display: inline-block;
    position: relative;
  }

  .list-box img {
    border-radius: 0;
  }

  .contain {
    object-fit: contain;
    object-position: center;
    height: 100%;
    width: 100%;
  }

  .cover {
    object-fit: cover;
  }

  .blur {
    filter: blur(15px);
    box-shadow: 1px 1px 2px 0;
  }

  .account-image {
    object-fit: cover;
    background-position: 50% 50%;
    border-radius: 50%;
    width: 160px;
    height: 160px;
  }

  :not(.list-box) > .collection-image {
    object-fit: cover;
    border-radius: 8px;
    width: 100%;
    height: 160px;
  }

  .list-box.search-image svg {
    margin: 10px;
  }
`,Ct=e`
  .waveform {
    mix-blend-mode: screen;
    position: relative;
    height: 62.5%;
    top: 50%;
    transform: translateY(-50%);
  }

  .waveform-grad0 {
    background: linear-gradient(
      hsl(300, 80%, 55%),
      hsl(330, 80%, 33%) 35%,
      hsl(330, 80%, 22%) 70%,
      hsl(0, 0%, 0%)
    );
  }

  .waveform-grad1 {
    background: linear-gradient(
      hsl(200, 80%, 55%),
      hsl(230, 80%, 33%) 35%,
      hsl(230, 80%, 22%) 70%,
      hsl(0, 0%, 0%)
    );
  }

  .waveform-grad2 {
    background: linear-gradient(
      hsl(160, 80%, 55%),
      hsl(190, 80%, 33%) 35%,
      hsl(190, 80%, 22%) 70%,
      hsl(0, 0%, 0%)
    );
  }

  .waveform-grad3 {
    background: linear-gradient(
      hsl(250, 80%, 55%),
      hsl(280, 80%, 33%) 35%,
      hsl(280, 80%, 22%) 70%,
      hsl(0, 0%, 0%)
    );
  }

  .waveform-grad4 {
    background: linear-gradient(
      hsl(280, 80%, 55%),
      hsl(310, 80%, 33%) 35%,
      hsl(310, 80%, 22%) 70%,
      hsl(0, 0%, 0%)
    );
  }

  .waveform-grad5 {
    background: linear-gradient(
      hsl(340, 80%, 55%),
      hsl(0, 80%, 33%) 35%,
      hsl(0, 80%, 22%) 70%,
      hsl(0, 0%, 0%)
    );
  }
`,wt=m`
  <svg
    viewBox="0 0 100 100"
    xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m17.0555551 41.3194459c0-12.7430552 10.3541664-23.1027772 23.0847216-23.1027772 12.7166664 0 23.0777773 10.359722 23.0777773 23.1027772 0 12.7361108-10.3611109 23.0986106-23.0777773 23.0986106-12.7305552 0-23.0847216-10.3624998-23.0847216-23.0986106zm-17.24305512 0c0 22.2916661 18.04583292 40.3472213 40.32777672 40.3472213 8.9208332 0 17.145833-2.9319449 23.8194439-7.8527776l24.1513883 24.0777771c1.2125 1.1402778 2.8430555 1.8430556 4.6374999 1.8430556 3.7444443 0 6.7805554-3.0361111 6.7805554-6.7791665 0-2.0652778-.9222222-3.9069444-2.3736111-5.1499999l-23.718055-23.7458328c4.4152777-6.4791665 7.0152776-14.3055552 7.0152776-22.7402772 0-22.2791661-18.0458328-40.34861006-40.312499-40.34861006-22.2819438 0-40.32777672 18.06944396-40.32777672 40.34861006z"
      fill="#2c2c2c"
      fill-rule="evenodd"
    />
  </svg>
`,Tt,Et,L=(Tt=class extends c{constructor(){super(...arguments),this.isListTile=!1,this.isCompactTile=!1,this.loggedIn=!1,this.suppressBlurring=!1,this.isWaveform=!1,this.isNotFound=!1}willUpdate(e){if(e.has(`model`)){let e=this.model?.identifier;this.isWaveform=Et.waveformByIdentifier.get(e)===!0}}render(){return a`
      <div class=${g(this.itemBaseClass)}>${this.imageTemplate}</div>
    `}get imageTemplate(){return this.model?.mediatype===`search`?a`${wt}`:a`
      <img
        class=${g(this.itemImageClass)}
        src="${this.imageSrc}"
        alt=""
        @load=${this.onLoad}
        @error=${this.onError}
      />
    `}get imageSrc(){if(this.isNotFound)return this.notFoundSrc;if(this.model?.captureDates&&this.model.identifier)try{let e=new URL(this.model.identifier),t=encodeURIComponent(e.hostname);return this.baseImageUrl?`https://web.archive.org/thumb/${t}?generate=1`:n}catch{return this.notFoundSrc}return this.model?.thumbnailUrl?this.model.thumbnailUrl:this.baseImageUrl&&this.model?.identifier?`${this.baseImageUrl}/services/img/${this.model.identifier}`:n}get notFoundSrc(){return this.baseImageUrl?`${this.baseImageUrl}/images/notfound.png`:n}get hashBasedGradient(){return this.model?.identifier?`waveform-grad${this.hashStrToInt(this.model.identifier)%6}`:`waveform-grad0`}hashStrToInt(e){return e.split(``).reduce((e,t)=>e+t.charCodeAt(0),0)}get itemBaseClass(){return{"drop-shadow":!0,"list-box":this.isListTile,"search-image":this.model?.mediatype===`search`,[this.hashBasedGradient]:this.isWaveform}}get itemImageClass(){let e=!!(this.model?.contentWarning||this.model?.loginRequired)&&!this.suppressBlurring;return{contain:!this.isCompactTile&&!this.isWaveform,cover:this.isCompactTile,blur:e,waveform:this.isWaveform,"account-image":this.isAccountImage,"collection-image":this.model?.mediatype===`collection`}}get isAccountImage(){return this.model?.mediatype===`account`&&!this.isCompactTile&&!this.isListTile}onLoad(){(this.model?.mediatype===`audio`||this.model?.mediatype===`etree`)&&this.baseImage.naturalWidth/this.baseImage.naturalHeight===4&&(this.isWaveform=!0,this.model?.identifier&&Et.waveformByIdentifier.set(this.model.identifier,!0))}onError(){this.isNotFound=!0}static get styles(){return[St,Ct,e`
        img {
          height: var(--imgHeight, 16rem);
          width: var(--imgWidth, 16rem);
        }

        .search-image {
          display: flex;
          align-items: center;
          justify-content: center;
          background: rgb(245, 245, 247);
          border-radius: 4px;
        }

        svg {
          height: 10rem;
          width: 10rem;
        }
      `]}},Et=Tt,Tt.waveformByIdentifier=new Map,Tt);C([t({type:Object})],L.prototype,`model`,void 0),C([t({type:String})],L.prototype,`baseImageUrl`,void 0),C([t({type:Boolean})],L.prototype,`isListTile`,void 0),C([t({type:Boolean})],L.prototype,`isCompactTile`,void 0),C([t({type:Boolean})],L.prototype,`loggedIn`,void 0),C([t({type:Boolean})],L.prototype,`suppressBlurring`,void 0),C([s()],L.prototype,`isWaveform`,void 0),C([s()],L.prototype,`isNotFound`,void 0),C([S(`img`)],L.prototype,`baseImage`,void 0),L=Et=C([x(`item-image`)],L);var Dt=m`
  <svg viewBox="0 0 100 100"
    xmlns="http://www.w3.org/2000/svg">
    <path
      d="m89.6854559 79.6500588c1.7300364 6.4823648 2.180423 13.3122689 3.3145441 20.3499412h-86c.5683151-15.8558542 2.98334063-30.7849367 15.1676149-41.6581341 22.9948067-20.518674 59.250299-9.0032844 67.517841 21.3081929zm-40.0998307-79.6500588c10.872402.0493248 19.9700408 9.25722341 19.917959 20.1421788-.0829413 11.042868-8.9616237 19.8492523-20.0602807 19.8578212-11.1181198 0-19.9397193-8.7904706-19.9397193-19.8908727-.0327543-11.11998815 9.0125781-20.17487063 20.082041-20.1091273z"
      fill="black"
      class="fill-color"
      fill-rule="evenodd"
    />
    <title>Icon of a person</title>
  </svg>
`,Ot=m`
  <svg viewBox="0 0 300 300"
    xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m225.568345 20c20.949641 0 28.431655 8.5714286 28.431655 28.5714286-.928802 21.2807881-.704608 42.5615763-.400898 63.8423644l.153512 10.640394c.13351 9.458128.247386 18.916256.247386 28.374384v100c0 18.571429-5.985612 28.571429-28.431655 28.571429-50.877698-1.428571-100.258992-1.428571-151.1366903 0-20.9496403 0-28.4316547-8.571429-28.4316547-28.571429v-202.8571424c0-18.5714286 5.9856115-28.5714286 28.4316547-28.5714286 50.8776983 1.4285714 100.2589923 1.4285714 151.1366903 0zm-74.268345 127.4c-30.251351 0-53.3 24.489189-53.3 53.3 0 30.251351 24.489189 53.3 53.3 53.3 30.251351 0 53.3-24.489189 53.3-53.3s-24.489189-53.3-53.3-53.3zm0 39c7.897672 0 14.3 6.402328 14.3 14.3s-6.402328 14.3-14.3 14.3-14.3-6.402328-14.3-14.3 6.402328-14.3 14.3-14.3zm-1.3-137.8c-20.384 0-36.4 16.016-36.4 36.4s16.016 36.4 36.4 36.4 36.4-16.016 36.4-36.4-17.472-36.4-36.4-36.4z"
      fill="black"
      fill-rule="evenodd"
    />
    <title>Icon of a speaker</title>
  </svg>
`,kt=m`
  <svg viewBox="0 0 100 100" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <title>icon/data(folder)</title>
    <g id="icon/data(folder)" stroke="none" stroke-fill="none" fill-rule="evenodd">
      <path
        class="fill-color"
        d="M100,41.0384365 L77.7420275,86 L0,86 L22.2588638,41.0384365 L100,41.0384365 Z M28.0641366,15 L28.1546605,15.0596181 L28.2341398,15.1242873 L28.3328904,15.2177465 L28.4472762,15.3436376 C28.6690939,15.606393 28.9514073,16.0510217 29.1933903,16.7785146 L29.1933903,16.7785146 L29.1933903,22.0756281 L78.7495321,22.0756281 L78.7495321,35.9042492 L17.4213444,35.9042492 L0.000356512594,72.0402693 L0.000356512594,23.7334897 L6.77320451,15 L28.0641366,15 Z"
        id="Combined-Shape"
        fill="#000000">
      </path>
    </g>
  </svg>
`,At=m`
  <svg viewBox="0 0 300 300"
    xmlns="http://www.w3.org/2000/svg">
    <path
      d="m136.125893 117.288073c14.41013-4.850562 27.83316-2.109766 39.480173 7.796632 10.335962 8.792594 14.330094 20.428621 14.288498 33.809481-.091795 25.476855-.015572 50.965113-.049872 76.441968-.003811 2.805419-.034301 5.648851-.457344 8.401051-3.376719 21.854139-21.133078 36.299386-43.127486 35.204588-20.210769-1.003564-37.307792-18.132586-37.799436-38.393946-.346818-14.513672-.060979-29.057755-.301084-43.586633 0-12.597776-.06479-25.191751.034301-37.789527.0343-3.656928.16007-7.401288.903253-10.963182 3.18997-15.36138 12.272049-25.963431 27.028997-30.920432zm69.082871-95.288073v20.8201628h-42.6892v27.2482979s2.61448.5740085 3.574902.8363038c23.797103 6.5155667 41.073252 15.8669636 55.262332 37.0824705 8.594246 12.852468 11.643202 31.380398 11.643202 46.760785v15.304359h-20.889162c0-5.538611.099091-10.750305-.038112-15.961998-.099091-3.896415-.167692-7.857454-.842274-11.689246-5.328051-30.262793-30.775402-49.8170956-57.026916-51.5315184-29.311903-1.9120945-57.1907972 18.1858064-64.1500399 46.8520184-1.6657464 6.843753-1.7799637 13.736569-1.7780383 20.63947l.0023262 1.88283c.0030489 1.883022.0050308 3.76653-.0231721 5.649734-.0228671 1.364696 0 2.72559 0 4.052272h-20.5918884c-5.5529117-51.542923 24.3078039-93.7420516 70.1221835-99.8660762v-27.3889489h-42.6205979v-20.6909159z"
      class="fill-color"
      fill="black"
      fill-rule="evenodd"
      transform="matrix(1 0 0 -1 0 301)"
    />
  </svg>
`,jt=m`
  <svg viewBox="0 0 300 300"
    xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m43.0952612 181.159979c2.1744514 0 21.7161099 3.336499 21.7161099 3.336499v21.030628l-44.8113711-6.289709c4.5382599-51.29161 9.6729948-105.5332879 14.6089046-156.237397 66.1329874 6.484496 144.5110704 16.1138469 211.0385514 22.4035567-.987813 6.4876377-1.581132 21.8160564-1.972471 28.3005524-4.939065-.5906421-15.599873 0-20.535783-.5906421v-9.0418506c-56.065498-5.3032118-112.326666-12.180422-168.3953197-17.6847035 0 0-7.7005244 74.2858081-11.6486211 114.7730661zm31.7867547-81.562016h205.1179841v158.402037h-205.1179841zm18.9514955 140.126691h167.8051566v-64.461671l-21.122791 35.963191-28.428821-67.408598-24.2819 28.891194-66.530637-54.634392h-27.4410076zm64.5550106-40.487257c0-11.394994-9.082832-20.436845-20.731453-20.436845-11.250971 0-20.923965 9.041851-20.923965 20.436845 0 11.203349 9.672994 20.439986 20.923965 20.439986 11.648621 0 20.731453-9.236637 20.731453-20.439986z"
      fill="black"
      transform="matrix(1 0 0 -1 0 301)"
    />
  </svg>
`,Mt=m`
  <svg viewBox="0 0 300 300" xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m272 280h-9.268431c.276116-.74734.440961-1.53597.440961-2.365889v-6.775607c0-3.831666-3.082608-6.920121-6.882294-6.920121h-12.701336c-3.795565 0-6.882294 3.100842-6.882294 6.920121v6.775607c0 .829919.164845 1.626807.432719 2.365889h-172.3623223c.2678738-.74734.4368403-1.53597.4368403-2.365889v-6.775607c0-3.831666-3.0826085-6.920121-6.8864155-6.920121h-12.6930937c-3.803807 0-6.8905366 3.100842-6.8905366 6.920121v6.775607c0 .829919.1689665 1.626807.4368402 2.365889h-11.1806374v-260h11.1806374c-.2678737.74734-.4368402 1.5359695-.4368402 2.3700175v6.7673495c0 3.8399238 3.0826085 6.9242496 6.8905366 6.9242496h12.6930937c3.7955647 0 6.8864155-3.0967127 6.8864155-6.9242496v-6.7673495c0-.8381769-.1689665-1.6309354-.4368403-2.3700175h172.3664433c-.271995.74734-.43684 1.5359695-.43684 2.3700175v6.7673495c0 3.8399238 3.078487 6.9242496 6.882294 6.9242496h12.701336c3.795565 0 6.882294-3.0967127 6.882294-6.9242496v-6.7673495c0-.8381769-.164845-1.6309354-.43684-2.3700175h9.26431zm-206.786157-222.1333969c0-3.8357948-3.0826085-6.9242496-6.8864155-6.9242496h-12.6930937c-3.803807 0-6.8905366 3.1008417-6.8905366 6.9242496v6.7714785c0 3.8399238 3.0826085 6.9283786 6.8905366 6.9283786h12.6930937c3.7955647 0 6.8864155-3.1008417 6.8864155-6.9283786zm0 35.5007147c0-3.8357949-3.0826085-6.9242497-6.8864155-6.9242497h-12.6930937c-3.803807 0-6.8905366 3.1008417-6.8905366 6.9242497v6.7714782c0 3.835795 3.0826085 6.920121 6.8905366 6.920121h12.6930937c3.7955647 0 6.8864155-3.096713 6.8864155-6.920121zm0 35.4965852c0-3.839923-3.0826085-6.924249-6.8864155-6.924249h-12.6930937c-3.803807 0-6.8905366 3.100841-6.8905366 6.924249v6.771479c0 3.839924 3.0826085 6.92425 6.8905366 6.92425h12.6930937c3.7955647 0 6.8864155-3.100842 6.8864155-6.92425zm0 35.500715c0-3.839924-3.0826085-6.92425-6.8864155-6.92425h-12.6930937c-3.803807 0-6.8905366 3.096713-6.8905366 6.92425v6.771479c0 3.839923 3.0826085 6.928378 6.8905366 6.928378h12.6930937c3.7955647 0 6.8864155-3.10497 6.8864155-6.928378zm0 35.496586c0-3.827537-3.0826085-6.92425-6.8864155-6.92425h-12.6930937c-3.803807 0-6.8905366 3.104971-6.8905366 6.92425v6.771478c0 3.839924 3.0826085 6.928379 6.8905366 6.928379h12.6930937c3.7955647 0 6.8864155-3.104971 6.8864155-6.928379zm0 35.500714c0-3.835794-3.0826085-6.924249-6.8864155-6.924249h-12.6930937c-3.803807 0-6.8905366 3.10497-6.8905366 6.924249v6.771479c0 3.835795 3.0826085 6.928378 6.8905366 6.928378h12.6930937c3.7955647 0 6.8864155-3.10497 6.8864155-6.928378zm155.247251-186.2734632c0-5.8672383-4.805242-10.6692075-10.673738-10.6692075h-119.5788334c-5.8684952 0-10.6737379 4.8019692-10.6737379 10.6692075v74.7009692c0 5.867238 4.8052427 10.665078 10.6737379 10.665078h119.5788334c5.868496 0 10.673738-4.79784 10.673738-10.665078zm.366781 121.3663652c0-5.90027-4.83409-10.727013-10.731434-10.727013h-120.1970034c-5.8973433 0-10.7273127 4.826743-10.7273127 10.727013v75.084961c0 5.896141 4.8299694 10.718755 10.7273127 10.718755h120.1970034c5.897344 0 10.731434-4.822614 10.731434-10.718755zm42.344655-112.5882169c0-3.8357948-3.082608-6.9242496-6.882294-6.9242496h-12.701336c-3.795565 0-6.882294 3.1008417-6.882294 6.9242496v6.7714785c0 3.8399238 3.078487 6.9283786 6.882294 6.9283786h12.701336c3.795565 0 6.882294-3.1008417 6.882294-6.9283786zm0 35.5007147c0-3.8357949-3.082608-6.9242497-6.882294-6.9242497h-12.701336c-3.795565 0-6.882294 3.1008417-6.882294 6.9242497v6.7714782c0 3.835795 3.078487 6.920121 6.882294 6.920121h12.701336c3.795565 0 6.882294-3.096713 6.882294-6.920121zm0 35.4965852c0-3.839923-3.082608-6.924249-6.882294-6.924249h-12.701336c-3.795565 0-6.882294 3.100841-6.882294 6.924249v6.771479c0 3.839924 3.078487 6.92425 6.882294 6.92425h12.701336c3.795565 0 6.882294-3.100842 6.882294-6.92425zm0 35.500715c0-3.839924-3.082608-6.92425-6.882294-6.92425h-12.701336c-3.795565 0-6.882294 3.096713-6.882294 6.92425v6.771479c0 3.839923 3.078487 6.928378 6.882294 6.928378h12.701336c3.795565 0 6.882294-3.10497 6.882294-6.928378zm0 35.496586c0-3.827537-3.082608-6.92425-6.882294-6.92425h-12.701336c-3.795565 0-6.882294 3.104971-6.882294 6.92425v6.771478c0 3.839924 3.078487 6.928379 6.882294 6.928379h12.701336c3.795565 0 6.882294-3.104971 6.882294-6.928379zm0 35.500714c0-3.835794-3.082608-6.924249-6.882294-6.924249h-12.701336c-3.795565 0-6.882294 3.10497-6.882294 6.924249v6.771479c0 3.835795 3.078487 6.928378 6.882294 6.928378h12.701336c3.795565 0 6.882294-3.10497 6.882294-6.928378z"
      fill="black"
      fill-rule="evenodd"
      transform="matrix(1 0 0 -1 0 300)"
    />
    <title>Icon of a film</title>
  </svg>
`,Nt=m`
  <svg viewBox="0 0 100 100" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <title>icon/radio</title>
    <g id="icon/radio" stroke="none" stroke-fill="none" fill-rule="evenodd">
      <path
        class="fill-color"
        d="M64.1512549,2.64739434 C66.1587977,1.39734524 68.798514,2.01257316 70.0484935,4.01892156 C71.2997791,6.02526996 70.6858916,8.66513331 68.6770426,9.9151824 C68.4249569,10.0719284 68.1637282,10.1999376 67.8959687,10.2979038 L67.8959687,10.2979038 L34.8074317,31.1764668 L35.0020471,31.1647109 C52.2287757,30.3692251 69.4411367,30.9217546 86.5973335,32.7295581 C88.6492852,32.94639 90.6947061,33.2494322 92.7335963,33.5903547 C96.8727656,34.2826494 99.9748567,37.8982564 99.9839997,42.1290913 C100.006204,58.2399645 100.003592,74.3443065 99.986612,90.4486486 C99.9800813,95.0478365 97.0242782,97.9084504 92.2411802,97.9175939 C87.5303554,97.9267375 82.8202563,97.9308012 78.1104474,97.9321558 L78.1104474,97.9321558 L49.8515942,97.9267375 C48.1320561,97.9267375 46.4123956,97.9245128 44.6926508,97.9210381 L44.6926508,97.9210381 L34.3734782,97.892985 C25.7738361,97.8701007 17.1740026,97.8646922 8.5787611,97.9985794 C3.01458916,98.0834834 -0.0731344263,94.3529294 0.00131576168,89.4284935 C0.129027585,80.8118193 0.140637751,72.1935325 0.126178794,63.5736331 L0.126178794,63.5736331 L0.0914450485,48.0560725 L0.0875212425,42.8827782 C0.0875212425,37.6931804 2.77164644,34.1128413 7.8695251,33.5328812 C10.8227159,33.1945711 13.7772128,32.8784668 16.7356282,32.578037 L16.7356282,32.578037 L64.1055399,2.57424622 Z M12.7505839,77.7247946 C11.0094944,77.7247946 9.46432649,78.2276879 9.5544504,80.4169066 C9.62881352,82.3029089 10.9422367,82.7552081 12.4614494,82.7996984 L12.4614494,82.7996984 L12.7897682,82.8033639 C15.5594458,82.7772396 18.3314092,82.7720147 21.1041889,82.773321 L21.1041889,82.773321 L29.4235076,82.7824645 L37.7418467,82.7723413 C40.5139733,82.7700554 43.2856101,82.773321 46.0572469,82.7955266 L46.0572469,82.7955266 L46.3771692,82.7934404 L46.6910226,82.7816532 C48.1384034,82.7002545 49.3222811,82.2230773 49.3291612,80.2457922 C49.333384,78.3566767 48.2339358,77.8765708 46.9366545,77.7629453 L46.9366545,77.7629453 L46.6341823,77.742755 C46.5833225,77.7403161 46.532273,77.7383006 46.4810855,77.7366494 L46.4810855,77.7366494 L46.1727164,77.7306375 L23.8394532,77.7342767 Z M72.5758814,48.314026 C63.8625972,48.2434903 56.222963,55.7385601 56.1119407,64.4732291 C55.9917755,73.5122467 63.4080592,81.0843832 72.4439609,81.1549188 C81.6522736,81.228067 88.9418613,74.0412643 89.0123931,64.8324387 C89.0816187,55.7268041 81.7802757,48.3950114 72.5758814,48.314026 Z M43.2828538,67.741865 L43.1158114,67.7413826 L38.8529055,67.7642822 C37.43176,67.7683233 36.0104716,67.769303 34.5891424,67.7688131 L34.5891424,67.7688131 L26.0614938,67.7609758 L14.6889495,67.7652331 L8.84172485,67.7569866 C7.40173575,67.7858956 6.34240993,68.3050616 6.30084658,70.1226151 C6.25251926,72.1341884 7.4175994,72.693249 9.04244209,72.693249 C20.4137287,72.681493 31.7863214,72.6736557 43.157608,72.6958614 L43.157608,72.6958614 L43.325337,72.6944626 C44.8758145,72.661732 45.9981125,72.069328 45.9881554,70.1800886 C45.976266,68.2181516 44.7445726,67.7283204 43.1158114,67.7413826 Z M72.582393,57.9750172 C76.6548848,58.0954055 79.8665352,61.5077032 79.7215514,65.56725 C79.5843346,69.4235606 75.9752732,72.8358582 72.2367619,72.645567 C68.1008397,72.4280913 65.0406456,69.125826 65.140322,64.9911982 C65.2425874,61.0455673 68.6251116,57.8507453 72.582393,57.9750172 Z M43.1249544,57.4692968 C41.2314815,57.4740863 39.3375733,57.475973 37.4433747,57.47636 L37.4433747,57.47636 L26.0758614,57.4719092 C20.3862997,57.4719092 14.7032687,57.4692968 9.02023765,57.4771341 C7.51033559,57.4771341 6.39227663,58.074075 6.30084658,59.902778 C6.20811038,61.7745861 7.29743418,62.4982299 8.81256081,62.5034548 C20.3301355,62.5230481 31.8450979,62.516517 43.3613664,62.5099859 C44.7693893,62.5034548 45.903122,61.9156574 45.9841029,60.2254134 C46.0820637,58.1393858 44.8490641,57.4692968 43.1249544,57.4692968 Z M46.2168861,47.4009001 L40.5790386,47.4140429 L29.6377151,47.4088181 L18.2454335,47.4123013 L12.5520501,47.4009808 C10.9546364,47.3931435 9.74384123,47.9548165 9.52571524,49.829237 C9.32848755,51.5364619 10.6202636,52.5618418 12.9582608,52.5657604 C23.9742763,52.5840474 34.9942102,52.5762101 46.0154503,52.5683728 L46.0154503,52.5683728 L46.1775177,52.5672124 L46.4993249,52.5570056 C47.9909315,52.4819668 49.3130496,51.9964186 49.3214617,49.9742271 C49.3331052,47.9543951 48.0608789,47.4733378 46.5450549,47.4087712 L46.5450549,47.4087712 L46.2168861,47.4009001 Z"
        id="Combined-Shape"
        fill="#000000">
      </path>
    </g>
  </svg>
`,Pt=m`
  <svg viewBox="0 0 300 300"
    xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m266.443951 20c8.716971 0 13.539477 4.8435881 13.556049 13.5985084v116.3828466 116.072094c0 9.214834-4.702358 13.946551-13.854348 13.946551h-232.4332033c-8.9489808 0-13.6969123-4.793868-13.6969123-13.768386-.0207152-77.476694-.0207152-154.961674 0-232.4590845 0-8.9662316 4.7479315-13.7725295 13.672054-13.7725295zm-197.7807608 138.512534c-2.3822518 0-5.1000904 1.052413-7.0887528 2.444582-4.1223314 2.871349-5.2575262 7.383468-5.2575262 12.227056l.0000647 20.524783c.0000093.952732.0000199 1.902825.0000319 2.850655l.0008306 25.31649c.0000546.937648.0001123 1.876427.0001735 2.816715l.0009089 11.379992c.0000914.958389.0001869 1.919795.0002867 2.884595l.0006526 5.832546c.0003539 2.939654.0007502 5.916643.0011941 8.941148 0 1.052413.0952901 2.092397.1574358 3.298115h183.648829l.28587-1.143568c.016572-29.633312.111862-55.119121-.033144-84.760721-.041431-7.391754-5.522681-12.703542-12.350422-12.703542-53.113858-.008287-106.236002-.045577-159.3664328.091154zm146.0755388-113.992128h-129.8596542l-.2444397 1.1601409c-.0082861 22.2001243-.0662888 44.3961053.0331443 66.6003731.0207153 5.676403 4.0228983 9.264553 9.7485888 9.264553 36.5333858.008287 73.0460558.008287 109.5835838.008287 7.391195 0 10.734634-3.372695 10.738777-10.876321zm-20.434335 9.1887301v53.7103789h-32.709353v-53.7103789z"
      fill="black"
      fill-rule="evenodd"
    />
  </svg>
`,Ft=m`
  <svg viewBox="0 0 300 300"
    xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m30.7264159 91.3202111 1.6974686 166.4298289s93.0569425-.896348 99.7147175 6.469589c4.752251 5.262329 29.15749 5.35494 31.185205 2.06061 5.607591-9.099099 85.824537-7.696693 102.729871-8.530199l1.905524-164.9480418 12.040798 4.2865984v175.2610164s-105.576598-1.796006-108.707338 4.190679c-.369876.714433-11.030243 3.459708-17.354469 3.459708l-.365424-.000017-.751764-.000432c-7.778071-.009122-19.543206-.203741-22.971013-4.355608-5.663733-6.863188-109.849992-4.187372-109.849992-4.187372v-175.5388508zm222.4119701-5.3202111v146.683693s-1.32429 4.845576-4.61685 8.004297c-2.777376 2.665893-8.834102 2.768428-8.834102 2.768428h-59.100966s-15.366384 3.883076-18.041383 8.146521l-7.21259-.046306v-156.044089c4.785276-5.1035658 12.024286-9.512544 22.929035-9.512544zm-132.891971 0c10.736323 0 17.929098 4.2766757 22.714375 9.2909375v156.2656955l-7.001233.046306c-5.165059-5.067182-18.044684-8.146521-18.044684-8.146521l-61.8453708-.000141c-.1987476-.00456-4.8027407-.135182-7.8201913-2.503683-2.7517631-2.168142-2.8740636-6.281222-2.8794992-6.642546l-.0002528-148.310048z"
      fill="black"
      fill-rule="evenodd"
    />
  </svg>
`,It=m`
  <svg viewBox="0 0 300 300"
    xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m237.917039 64.413845c0-9.2252653-7.280839-16.7597371-16.145202-16.7597371h-143.5076619c-8.8913162 0-16.1755245 7.5344718-16.1755245 16.7597371v90.110532c0 9.218529 7.2842083 16.756369 16.1755245 16.756369h143.5076619c8.864363 0 16.145202-7.53784 16.145202-16.756369zm26.070862 112.508493c.347027 17.780275-6.761982 22.667409-22.50288 21.923056-19.861428-.943072-39.773394-.2324-59.668515-.2324h-3.584827c0 .121252.067384.222296.067384.343548 0 5.530444-3.416368 10.161601-8.163569 11.970278l34.729703 56.607899c2.368547 3.859859 1.165743 8.905294-2.712205 11.246134-3.867841 2.37452-8.894686 1.175471-11.243017-2.684388l-39.436475-64.283831h-7.280839l-37.556461 61.255897c-2.378655 3.856491-7.3987611 5.055539-11.2666016 2.687755-3.8577329-2.35768-5.0639061-7.403116-2.708836-11.259607l34.0558636-55.503156c-2.951418-2.344208-5.003261-5.884096-5.003261-10.036981 0-.134725.064015-.242504.064015-.370493-26.0169545-.07073-45.1472665-.740985-64.1124877.296394-16.711227.879078-21.9402239-6.069342-21.653842-22.839183.8153462-44.310913.7917618-88.6218257.033692-132.9327384-.2931203-16.5138648 4.4945115-23.3275602 21.3775677-23.1052646 61.993262.7544576 124.003369.6332055 186.023585.0505217 15.302902-.1481971 20.723943 5.6011742 20.477991 21.3572132-.636778 45.1563095-.808608 90.3564041.064015 135.5093461z"
      fill="black"
      fill-rule="evenodd"
      transform="matrix(1 0 0 -1 0 300)"
    />
  </svg>
`,Lt=a`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m38.4615385 55.5197275-4.8171501 7.0895871c4.1025641 2.9421734 8.9186633 4.7253407 14.4482976 5.3495018v13.0411836h5.6179067v-12.9749847c3.8356452-.357264 6.8905002-1.5278287 9.1645649-3.5116942 2.2740648-1.9838655 3.4110971-4.5582672 3.4110971-7.7232052v-.0661989c0-3.2553047-1.025641-5.7635075-3.076923-7.5246085-2.0512821-1.7611009-5.3068517-3.1544302-9.7667087-4.1799878v-6.2195444c2.6754939.6682937 5.083018 1.7379839 7.2225725 3.2090706l4.4136191-7.1557861c-3.2114333-2.2738797-6.9798234-3.6787675-11.3051702-4.2146634v-11.6383974h-5.5517024v11.5028473c-4.0584279.3131313-7.2026061 1.4721374-9.4325347 3.4770185-2.2299285 2.004881-3.3448928 4.5908413-3.3448928 7.7578808v.1323978c0 3.2111721 1.0372005 5.6857501 3.1116015 7.423734 2.074401 1.7379838 5.3625473 3.1197546 9.8644389 4.1453122v6.4212934c-3.1651955-.4896617-6.4869694-1.9386821-9.9653216-4.3470612zm-24.4136192-32.7716087c-4.32534679 3.7008338-7.74800337 8.0142702-10.26796974 12.9403091s-3.77994956 10.1536506-3.77994956 15.6828351c0 4.9050234 1.02564103 9.6093485 3.07692308 14.1129754 2.05128205 4.5036268 4.88335435 8.5606738 8.49621692 12.1711409.357293.3572639 1.1149642.6252118 2.2730138.8038438s2.2057588-.0220663 3.1431274-.6020948c.5800757-.4896617.8701135-1.1138228.8701135-1.8724832s-.1229508-1.3828215-.3688525-1.8724832c-.2459016-.4896617-.5023119-.8689919-.7692307-1.1379907-6.5111392-6.3761101-9.76670873-13.5770795-9.76670873-21.6029082 0-4.6822589 1.1034048-9.0850112 3.31021443-13.2082571 2.2068095-4.1232459 5.2059688-7.6570063 8.9974779-10.6012813.0882724-.0882652.1775956-.1880889.2679697-.2994712s.2574611-.3562131.5012611-.7344926c.2437999-.3782794.4224464-.7460511.5359394-1.103315.1134931-.3572639.1250526-.770219.0346785-1.2388652s-.3131568-.8816012-.6683481-1.2388651c-.668348-.6682937-1.4932745-1.0245068-2.4747793-1.0686394s-1.8064313.0893159-2.4747793.4003457zm71.9041614 0c4.3253468 3.7008338 7.7480034 8.0142702 10.2679697 12.9403091 2.5199664 4.9260389 3.7799496 10.1536506 3.7799496 15.6828351 0 4.8608908-1.025641 9.5536573-3.0769231 14.0782997-2.051282 4.5246424-4.8602354 8.593248-8.42686 12.2058166-.4014292.3572639-1.1822194.6252118-2.3423707.8038438-1.1601514.178632-2.1857924-.0220663-3.0769231-.6020948-.5800757-.4896617-.8806221-1.1138228-.9016394-1.8724832-.0210172-.7586604.0903741-1.3828215.3341741-1.8724832.2437999-.4896617.5002101-.8689919.7692307-1.1379907 6.4649012-6.3319775 9.6973519-13.5329469 9.6973519-21.6029082 0-4.6822589-1.1034048-9.0850112-3.3102144-13.2082571s-5.18285-7.6570063-8.9281211-10.6012813c-.0882724-.0882652-.1775956-.1880889-.2679697-.2994712s-.2574611-.3562131-.5012611-.7344926c-.2437999-.3782794-.4224464-.7460511-.5359394-1.103315-.1134931-.3572639-.1250526-.770219-.0346785-1.2388652s.3131568-.8816012.6683481-1.2388651c.668348-.6682937 1.4932745-1.0245068 2.4747793-1.0686394s1.7843632.0893159 2.408575.4003457zm-62.2730139 8.0920276c-3.1210592 2.4525117-5.6179066 5.4840011-7.4905422 9.0944682s-2.8089534 7.4447495-2.8089534 11.5028473c0 6.2878449 2.2068096 11.9725442 6.6204288 17.054098.1786465.3131313.858554.469697 2.0397226.469697 1.1811685 0 2.1290458-.2900143 2.8436318-.8700427.5359394-.4896618.8259773-1.1253814.8701134-1.9071589.0441362-.7817775-.045187-1.2167988-.2679697-1.305064-1.6939891-1.7842181-3.1094998-3.9582741-4.2465321-6.522168-1.1370324-2.563894-1.7055486-5.0720968-1.7055486-7.5246085 0-3.34357.7471627-6.3424853 2.241488-8.996746 1.4943254-2.6542607 3.5571669-4.7390008 6.1885246-6.2542201.0441362-.0441326.1113914-.12189.2017655-.2332723s.2238335-.3341468.4003783-.6682937c.1765447-.3341468.2994956-.6577859.3688524-.9709172.0693569-.3131313.0472888-.7029693-.0662043-1.1695139-.113493-.4665447-.3478352-.901566-.7030264-1.3050641-.4897016-.5358959-1.102354-.8364179-1.8379572-.901566s-1.3482555-.009457-1.8379571.1670734zm52.5063052 0c3.1651955 2.4966443 5.6956705 5.5396923 7.591425 9.1291438 1.8957545 3.5894516 2.8436318 7.4132262 2.8436318 11.471324 0 6.2416107-2.2519967 11.9263101-6.75599 17.054098-.1345103.3131313-.7923497.469697-1.9735182.469697-1.1811686 0-2.1290459-.2900142-2.8436318-.8700427-.5359395-.4896617-.8259773-1.1253813-.8701135-1.9071589-.0441362-.7817775.045187-1.2167988.2679697-1.305064 1.6498529-1.8283506 3.0432955-4.0244729 4.1803279-6.5883669s1.7055485-5.0500305 1.7055485-7.4584096c0-3.3435699-.7356032-6.3309267-2.2068095-8.9620704-1.4712064-2.6311436-3.5004204-4.7263914-6.0876419-6.2857433l-.2332913-.2332723s-.1450189-.2227646-.4350568-.6682937c-.2900378-.4455291-.4129886-.7691682-.3688524-.9709172.0441362-.2017491.0662043-.591587.0662043-1.169514 0-.5779269.2343421-1.0129482.7030265-1.305064.4897015-.5358959 1.1023539-.8364179 1.8379571-.901566.7356032-.0651482 1.3482555-.009457 1.8379571.1670734zm-31.3682219 10.2324588v-.0661989c0-1.605586 1.2263556-2.497695 3.6790668-2.676327v5.7529998c-1.4270702-.4013965-2.3970156-.8248593-2.9098361-1.2703884-.5128205-.4455292-.7692307-1.0255576-.7692307-1.7400855zm12.1721311 16.3196055v.0661989c0 1.5614535-1.2042875 2.4756288-3.6128625 2.7425259v-5.8853976c1.382934.4013965 2.3308112.8248594 2.8436317 1.2703885.5128206.4455291.7692308 1.0476239.7692308 1.8062843z"
      fill="black"
      fill-rule="evenodd"
    />
  </svg>
`,Rt=a`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m53.8329111 72.530691-28.3707393-20.2314977c-1.2950216-.9306676-2.0122823-2.1279084-2.1517819-3.5917222-.1394997-1.4638139.1499621-2.9054133.8683852-4.3247984.7184232-1.419385 1.5275211-2.780311 2.4272938-4.0827779.8997726-1.302467 1.7204955-2.3488835 2.4621686-3.1392494l1.178772-1.1855489 20.6040968 16.1873027 36.4861309-43.88284817 3.885065-3.27955103c2.311044 1.53396469 4.1722017 3.06909856 5.5834731 4.60540161 1.4112713 1.53630309 2.2901191 2.86215359 2.6365432 3.97755169.3464241 1.1153982.5312611 2.1150474.5545111 2.9989478.0232494.8839004-.0813748 1.5351339-.3138742 1.9537004l-.4150115.6980007zm15.0520112 11.0943528v-5.022799c-.0464999-.1403016-.0813748-.2911259-.1046247-.4524728-.02325-.1613469-.0464999-.486379-.0697498-.9750964-.02325-.4887175.0116249-.8020578.1046247-.9400211.0464999-.1870689.2197119-.4898866.5196361-.9084532.2999243-.4185666.589386-.7903659.8683853-1.1153981l.4150114-.4875482 9.5731628-11.1645037v32.4412487h-80.1913681v-80.6524027h68.1874241l-7.1458691 8.5829534c-1.0183473 1.5807319-1.8041953 2.5113995-2.3575439 2.7920028h-47.3775653v57.9059979h57.574989z"
      fill="black"
      fill-rule="evenodd"
    />
  </svg>
`,zt=m`
  <svg viewBox="0 0 300 300"
    xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m272 280h-9.268431c.276116-.74734.440961-1.53597.440961-2.365889v-6.775607c0-3.831666-3.082608-6.920121-6.882294-6.920121h-12.701336c-3.795565 0-6.882294 3.100842-6.882294 6.920121v6.775607c0 .829919.164845 1.626807.432719 2.365889h-172.3623223c.2678738-.74734.4368403-1.53597.4368403-2.365889v-6.775607c0-3.831666-3.0826085-6.920121-6.8864155-6.920121h-12.6930937c-3.803807 0-6.8905366 3.100842-6.8905366 6.920121v6.775607c0 .829919.1689665 1.626807.4368402 2.365889h-11.1806374v-260h11.1806374c-.2678737.74734-.4368402 1.5359695-.4368402 2.3700175v6.7673495c0 3.8399238 3.0826085 6.9242496 6.8905366 6.9242496h12.6930937c3.7955647 0 6.8864155-3.0967127 6.8864155-6.9242496v-6.7673495c0-.8381769-.1689665-1.6309354-.4368403-2.3700175h172.3664433c-.271995.74734-.43684 1.5359695-.43684 2.3700175v6.7673495c0 3.8399238 3.078487 6.9242496 6.882294 6.9242496h12.701336c3.795565 0 6.882294-3.0967127 6.882294-6.9242496v-6.7673495c0-.8381769-.164845-1.6309354-.43684-2.3700175h9.26431zm-206.786157-222.1333969c0-3.8357948-3.0826085-6.9242496-6.8864155-6.9242496h-12.6930937c-3.803807 0-6.8905366 3.1008417-6.8905366 6.9242496v6.7714785c0 3.8399238 3.0826085 6.9283786 6.8905366 6.9283786h12.6930937c3.7955647 0 6.8864155-3.1008417 6.8864155-6.9283786zm0 35.5007147c0-3.8357949-3.0826085-6.9242497-6.8864155-6.9242497h-12.6930937c-3.803807 0-6.8905366 3.1008417-6.8905366 6.9242497v6.7714782c0 3.835795 3.0826085 6.920121 6.8905366 6.920121h12.6930937c3.7955647 0 6.8864155-3.096713 6.8864155-6.920121zm0 35.4965852c0-3.839923-3.0826085-6.924249-6.8864155-6.924249h-12.6930937c-3.803807 0-6.8905366 3.100841-6.8905366 6.924249v6.771479c0 3.839924 3.0826085 6.92425 6.8905366 6.92425h12.6930937c3.7955647 0 6.8864155-3.100842 6.8864155-6.92425zm0 35.500715c0-3.839924-3.0826085-6.92425-6.8864155-6.92425h-12.6930937c-3.803807 0-6.8905366 3.096713-6.8905366 6.92425v6.771479c0 3.839923 3.0826085 6.928378 6.8905366 6.928378h12.6930937c3.7955647 0 6.8864155-3.10497 6.8864155-6.928378zm0 35.496586c0-3.827537-3.0826085-6.92425-6.8864155-6.92425h-12.6930937c-3.803807 0-6.8905366 3.104971-6.8905366 6.92425v6.771478c0 3.839924 3.0826085 6.928379 6.8905366 6.928379h12.6930937c3.7955647 0 6.8864155-3.104971 6.8864155-6.928379zm0 35.500714c0-3.835794-3.0826085-6.924249-6.8864155-6.924249h-12.6930937c-3.803807 0-6.8905366 3.10497-6.8905366 6.924249v6.771479c0 3.835795 3.0826085 6.928378 6.8905366 6.928378h12.6930937c3.7955647 0 6.8864155-3.10497 6.8864155-6.928378zm155.247251-186.2734632c0-5.8672383-4.805242-10.6692075-10.673738-10.6692075h-119.5788334c-5.8684952 0-10.6737379 4.8019692-10.6737379 10.6692075v74.7009692c0 5.867238 4.8052427 10.665078 10.6737379 10.665078h119.5788334c5.868496 0 10.673738-4.79784 10.673738-10.665078zm.366781 121.3663652c0-5.90027-4.83409-10.727013-10.731434-10.727013h-120.1970034c-5.8973433 0-10.7273127 4.826743-10.7273127 10.727013v75.084961c0 5.896141 4.8299694 10.718755 10.7273127 10.718755h120.1970034c5.897344 0 10.731434-4.822614 10.731434-10.718755zm42.344655-112.5882169c0-3.8357948-3.082608-6.9242496-6.882294-6.9242496h-12.701336c-3.795565 0-6.882294 3.1008417-6.882294 6.9242496v6.7714785c0 3.8399238 3.078487 6.9283786 6.882294 6.9283786h12.701336c3.795565 0 6.882294-3.1008417 6.882294-6.9283786zm0 35.5007147c0-3.8357949-3.082608-6.9242497-6.882294-6.9242497h-12.701336c-3.795565 0-6.882294 3.1008417-6.882294 6.9242497v6.7714782c0 3.835795 3.078487 6.920121 6.882294 6.920121h12.701336c3.795565 0 6.882294-3.096713 6.882294-6.920121zm0 35.4965852c0-3.839923-3.082608-6.924249-6.882294-6.924249h-12.701336c-3.795565 0-6.882294 3.100841-6.882294 6.924249v6.771479c0 3.839924 3.078487 6.92425 6.882294 6.92425h12.701336c3.795565 0 6.882294-3.100842 6.882294-6.92425zm0 35.500715c0-3.839924-3.082608-6.92425-6.882294-6.92425h-12.701336c-3.795565 0-6.882294 3.096713-6.882294 6.92425v6.771479c0 3.839923 3.078487 6.928378 6.882294 6.928378h12.701336c3.795565 0 6.882294-3.10497 6.882294-6.928378zm0 35.496586c0-3.827537-3.082608-6.92425-6.882294-6.92425h-12.701336c-3.795565 0-6.882294 3.104971-6.882294 6.92425v6.771478c0 3.839924 3.078487 6.928379 6.882294 6.928379h12.701336c3.795565 0 6.882294-3.104971 6.882294-6.928379zm0 35.500714c0-3.835794-3.082608-6.924249-6.882294-6.924249h-12.701336c-3.795565 0-6.882294 3.10497-6.882294 6.924249v6.771479c0 3.835795 3.078487 6.928378 6.882294 6.928378h12.701336c3.795565 0 6.882294-3.10497 6.882294-6.928378z"
      fill="black"
      fill-rule="evenodd"
      transform="matrix(1 0 0 -1 0 300)"
    />
  </svg>
`,Bt=m`
  <svg viewBox="0 0 300 300"
    xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m261.437982 261.890554h-28.778763v-28.083112h28.778763zm-42.442013 0h-28.793735v-28.083112h28.793735zm40.645399-150.356298v75.703472 19.821613h-219.2617757v-19.821613-75.703472zm-84.228262 150.356298h-134.608315v-27.499419h134.608315zm-155.413106-169.890554v87.643734 100.356266h260v-100.356266-87.643734z"
      fill="black"
      transform="matrix(1 0 0 -1 0 372)"
    />
  </svg>
`,Vt=a`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      class="fill-color"
      d="m45.3394669 100.000002h7.4385828c6.4511217-.3595144 12.58904-1.9152224 18.4137548-4.6671261s10.8424956-6.3426695 15.0533425-10.7722976c4.2108468-4.429628 7.556034-9.6360206 10.0355616-15.6191778s3.7192914-12.2669975 3.7192914-18.8515208c0-9.0967521-2.2250498-17.4897316-6.6751493-25.1789383-4.4500995-7.6892068-10.5140669-13.7638762-18.1919023-18.22400833-7.6778355-4.46013212-16.0560287-6.68910874-25.1345797-6.68693148-6.8100007 0-13.2915728 1.31603472-19.4447163 3.94810096-6.1531434 2.63206624-11.4699901 6.17816545-15.9505399 10.63829755-4.4805499 4.4601321-8.03453942 9.7863622-10.66196865 15.9786902s-3.94114385 12.7005912-3.94114385 19.5247894c0 12.9271929 4.36092352 24.1624988 13.0827705 33.7059178 8.7218471 9.543419 19.4751667 14.9448198 32.2599589 16.2042045zm-28.136113-51.5246596c2.2707252-7.480036 5.6159125-13.1058597 10.0355616-16.8774711 4.2412972-3.9502783 8.9610997-5.4166529 14.1594076-4.399124.2979783.1786667.1783519.507675-.358879.9870248-.537231.4793498-1.433341 1.2572039-2.6883299 2.3335621-1.254989 1.0763582-2.3000881 2.1538058-3.1352974 3.2323429-1.074462 1.4358706-1.7911325 2.7976598-2.1500114 4.0853677-.358879 1.2877079-.448055 2.214814-.267528 2.7813184.180527.5665043.180527 1.1798542 0 1.8400496 3.1059345-.1786668 5.8236273.5545206 8.1530782 2.199562s3.9726817 3.7095139 4.9296923 6.1934174c.9570107 2.4839036 1.210401 5.2063926.760171 8.1674672-.45023 2.9610745-1.6595435 5.6094822-3.6279404 7.9452231-3.3451872 3.6495952-7.5712592 5.2205552-12.678216 4.7128802s-8.9154242-2.6767329-11.4254021-6.5071737c-1.5529673-2.1548953-2.4490773-4.8479697-2.6883299-8.0792232-.2392527-3.2312535.089176-6.1040841.9852859-8.6184917zm34.7655868 0c2.2098245-7.3013692 5.5854621-12.9271929 10.1269126-16.8774711 4.1825715-3.9502783 8.8719237-5.4166529 14.0680565-4.399124.358879.1786667.2838407.507675-.225115.9870248-.5089556.4793498-1.3898404 1.2572039-2.6426544 2.3335621-1.2528139 1.0763582-2.3283634 2.1538058-3.2266484 3.2323429-1.0744619 1.4358706-1.7911324 2.7976598-2.1500114 4.0853677s-.4632801 2.214814-.3132035 2.7813184c.1500767.5665043.1348515 1.1798542-.0456755 1.8400496 3.1059346-.1786668 5.8236273.5545206 8.1530782 2.199562s3.9726817 3.7095139 4.9296924 6.1934174c.9570106 2.4839036 1.2256261 5.2063926.8058464 8.1674672-.4197796 2.9610745-1.6443183 5.6094822-3.6736158 7.9452231-3.3451873 3.6495952-7.5712593 5.2205552-12.6782161 4.7128802s-8.9154242-2.6767329-11.4254021-6.5071737c-1.4942416-2.1548953-2.3609888-4.8479697-2.6002414-8.0792232-.2392527-3.2312535.0598131-6.1040841.8971975-8.6184917z"
      fill="black"
      fill-rule="evenodd"
    />
  </svg>
`,Ht={account:{color:`#000000`,icon:Dt,get text(){return o(`Account`)}},audio:{color:`#00adef`,icon:Ot,get text(){return o(`Audio`)}},collection:{color:`#4666ff`,icon:ct,get text(){return o(`Collection`)}},data:{color:`#333333`,icon:kt,get text(){return o(`Data`)}},etree:{color:`#00adef`,icon:At,get text(){return o(`E-tree`)}},film:{color:`#bf1b2c`,icon:Mt,get text(){return o(`Film`)}},image:{color:`#aa99c9`,icon:jt,get text(){return o(`Image`)}},movies:{color:`#f1644b`,icon:Mt,get text(){return o(`Movie`)}},none:{color:`#00000000`,icon:a``,text:``},radio:{color:`#8fdaef`,icon:Nt,get text(){return o(`Radio`)}},software:{color:`#9ecc4f`,icon:Pt,get text(){return o(`Software`)}},texts:{color:`#faab3c`,icon:Ft,get text(){return o(`Text`)}},tv:{color:`#f1644b`,icon:It,get text(){return o(`TV`)}},tvCommercial:{color:`#84b648`,icon:Lt,get text(){return o(`TV Political Ad`)}},tvFactCheck:{color:`#f1644b`,icon:Rt,get text(){return o(`TV Fact Check`)}},tvQuote:{color:`#fe7a5f`,icon:Vt,get text(){return o(`TV Quote`)}},video:{color:`#f1644b`,icon:zt,get text(){return o(`Video`)}},web:{color:`#ffcd27`,icon:Bt,get text(){return o(`Web`)}},search:{color:`#000000`,icon:wt,get text(){return o(`Search`)}}},Ut=`tv_ads`,Wt=`factchecked`,Gt=new Set([`tvnews`,`tvarchive`,`television`]),Kt=new Set([`radio`,`radioprogram`]),qt=class extends c{constructor(){super(...arguments),this.showText=!1}get displayMediatype(){return this.isTvItem?this.tvDisplayMediatype:this.isRadioItem?`radio`:this.model?.mediatype??`none`}get tvDisplayMediatype(){return this.isTvCommercial?`tvCommercial`:this.model?.isTvSearchResult&&this.isTvFactCheck?`tvFactCheck`:this.model?.isTvSearchResult&&this.isTvQuote?`tvQuote`:`tv`}get isTvItem(){return!!(this.model?.mediatype===`movies`&&this.model?.collections.some(e=>Gt.has(e)))}get isTvCommercial(){return!!(this.model?.adIds?.length||this.model?.collections.includes(Ut))}get isTvFactCheck(){return!!(this.model?.factChecks?.length||this.model?.collections.includes(Wt))}get isTvQuote(){return!!this.model?.isClip}get isRadioItem(){return!!(this.model?.mediatype===`audio`&&this.model?.collections.some(e=>Kt.has(e)))}render(){let e=Ht[this.displayMediatype];return e?a`
      <div
        id="icon"
        class="${this.showText?`show-text`:`hide-text`}"
        title="${e.text}"
        style="--iconFillColor: ${e.color}"
      >
        ${e.icon}
        <p class="status-text">${e.text}</p>
      </div>
    `:a``}static get styles(){return e`
      #icon {
        height: var(--iconHeight, 25px);
      }

      .status-text {
        font-size: 14px;
        color: #2c2c2c;
        margin: auto;
        display: block;
        text-align: var(--iconTextAlign, center);
      }

      #icon.hide-text p {
        display: none;
      }

      svg {
        height: var(--iconHeight, 10px);
        width: var(--iconWidth, 10px);
        pointer-events: none;
      }

      .fill-color {
        fill: var(--iconFillColor, '#000000');
      }
    `}};C([t({type:Object})],qt.prototype,`model`,void 0),C([t({type:Boolean})],qt.prototype,`showText`,void 0),qt=C([x(`tile-mediatype-icon`),i()],qt);var Jt=m`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      d="m100 7.78013601c0-2.14552613-.7593357-3.978597-2.278007-5.4992126-1.5186713-1.52061561-3.3493984-2.28092341-5.4921813-2.28092341h-84.54977287c-2.08268321 0-3.88336049.7603078-5.40203183 2.28092341-1.51867133 1.5206156-2.278007 3.35368647-2.278007 5.4992126v51.49262709c0 2.0853495.75933567 3.8883321 2.278007 5.4089477 1.51867134 1.5206156 3.31934862 2.2809234 5.40203183 2.2809234h10.53361537l.3571304 33.0373658 32.4087237-33.0373658h41.2468361c2.1427829 0 3.97351-.7603078 5.4921813-2.2809234s2.278007-3.3235982 2.278007-5.4089477z"
      fill="#333"
    />
    <title>Icon of a speech bubble</title>
  </svg>
`,Yt=m`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      d="m50 20 33.3333333 43.3333333h-20v36.6666667h-26.6666666v-36.6666667h-20zm50-20v13.3333333h-100v-13.3333333z"
      fill="#333"
      fill-rule="evenodd"
    />
    <title>Icon of an arrow pointing upwards</title>
  </svg>
`,Xt=m`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      d="m98 50.5704143c-.2830293-.471515-.671154-1.1088947-1.1643742-1.9121392s-1.6003642-2.3617474-3.3214321-4.6755089c-1.7210678-2.3137614-3.522258-4.5325939-5.4035703-6.6564975-1.8813124-2.1239037-4.2828993-4.473133-7.2047606-7.0476881-2.9218612-2.5745551-5.8895067-4.7933876-8.9029363-6.6564976-3.0134295-1.86311-6.4628491-3.4330878-10.3482587-4.7099336-3.8854095-1.2768458-7.7822651-1.9142256-11.6905667-1.9121443-3.9083017.0020914-7.8051573.6154781-11.6905668 1.8401652-3.8854096 1.2246871-7.3702078 2.8301329-10.4543947 4.8163375-3.0841869 1.9862045-6.0278997 4.1695691-8.8311384 6.5500937s-5.2048256 4.7652219-7.2047605 7.1540919c-1.99993501 2.38887-3.75430043 4.5722346-5.26309632 6.5500938s-2.63883199 3.583305-3.39010829 4.8163374l-1.13003609 1.8401602c.2830293.4715149.67115403 1.1088946 1.16437421 1.9121391.49322017.8032445 1.5878776 2.3617475 3.28397229 4.6755089s3.47439274 4.521119 5.3348942 6.6220728c1.8605014 2.1009538 4.2506422 4.4387083 7.1704224 7.0132633 2.9197801 2.5745551 5.8874256 4.7819127 8.9029363 6.6220729 3.0155106 1.8401601 6.4774168 3.398663 10.3857184 4.6755088 3.9083017 1.2768458 7.8176438 1.9142256 11.7280266 1.9121443 3.9103827-.0020914 7.7957922-.6154781 11.6562286-1.8401652s7.3337886-2.818658 10.4200566-4.7819127 6.0299808-4.1351444 8.8311384-6.515669 5.2152311-4.7652219 7.2422203-7.1540919 3.8052873-4.5607597 5.3348942-6.515669c1.5296068-1.9549093 2.6721295-3.5488802 3.427568-4.7819127zm-24.5142913 0c0 6.467683-2.3079374 12.0152859-6.9238123 16.6428087s-10.1495139 6.9412843-16.600917 6.9412843c-6.4992683 0-12.0453939-2.3137615-16.6383767-6.9412843s-6.8894742-10.1751257-6.8894742-16.6428087 2.2964914-12.003811 6.8894742-16.608384 10.1391084-6.9068595 16.6383767-6.9068595c6.4534842 0 11.9871232 2.3022865 16.600917 6.9068595s6.9217312 10.140701 6.9238123 16.608384zm-23.5247293-10.552755c2.8261308 0 5.2870289 1.0619518 7.3826944 3.1858555 2.0956655 2.1239036 3.1434982 4.5795368 3.1434982 7.3668995 0 2.8332624-1.0478327 5.2888956-3.1434982 7.3668995-2.0956655 2.078004-4.5565636 3.1170059-7.3826944 3.1170059-2.873996 0-5.3348941-1.0264838-7.3826944-3.0794516-2.0478002-2.0529677-3.0717003-4.5200758-3.0717003-7.4013243 0-2.8332624 1.0239001-5.3003705 3.0717003-7.4013243 2.0478003-2.1009538 4.5086984-3.1514307 7.3826944-3.1514307z"
      fill="#333"
    />
    <title>Eye icon</title>
  </svg>
`,Zt=m`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path d="m45.3394669 100.000002h7.4385828c6.4511217-.3595144 12.58904-1.9152224 18.4137548-4.6671261s10.8424956-6.3426695 15.0533425-10.7722976c4.2108468-4.429628 7.556034-9.6360206 10.0355616-15.6191778s3.7192914-12.2669975 3.7192914-18.8515208c0-9.0967521-2.2250498-17.4897316-6.6751493-25.1789383-4.4500995-7.6892068-10.5140669-13.7638762-18.1919023-18.22400833-7.6778355-4.46013212-16.0560287-6.68910874-25.1345797-6.68693148-6.8100007 0-13.2915728 1.31603472-19.4447163 3.94810096-6.1531434 2.63206624-11.4699901 6.17816545-15.9505399 10.63829755-4.4805499 4.4601321-8.03453942 9.7863622-10.66196865 15.9786902s-3.94114385 12.7005912-3.94114385 19.5247894c0 12.9271929 4.36092352 24.1624988 13.0827705 33.7059178 8.7218471 9.543419 19.4751667 14.9448198 32.2599589 16.2042045zm-28.136113-51.5246596c2.2707252-7.480036 5.6159125-13.1058597 10.0355616-16.8774711 4.2412972-3.9502783 8.9610997-5.4166529 14.1594076-4.399124.2979783.1786667.1783519.507675-.358879.9870248-.537231.4793498-1.433341 1.2572039-2.6883299 2.3335621-1.254989 1.0763582-2.3000881 2.1538058-3.1352974 3.2323429-1.074462 1.4358706-1.7911325 2.7976598-2.1500114 4.0853677-.358879 1.2877079-.448055 2.214814-.267528 2.7813184.180527.5665043.180527 1.1798542 0 1.8400496 3.1059345-.1786668 5.8236273.5545206 8.1530782 2.199562s3.9726817 3.7095139 4.9296923 6.1934174c.9570107 2.4839036 1.210401 5.2063926.760171 8.1674672-.45023 2.9610745-1.6595435 5.6094822-3.6279404 7.9452231-3.3451872 3.6495952-7.5712592 5.2205552-12.678216 4.7128802s-8.9154242-2.6767329-11.4254021-6.5071737c-1.5529673-2.1548953-2.4490773-4.8479697-2.6883299-8.0792232-.2392527-3.2312535.089176-6.1040841.9852859-8.6184917zm34.7655868 0c2.2098245-7.3013692 5.5854621-12.9271929 10.1269126-16.8774711 4.1825715-3.9502783 8.8719237-5.4166529 14.0680565-4.399124.358879.1786667.2838407.507675-.225115.9870248-.5089556.4793498-1.3898404 1.2572039-2.6426544 2.3335621-1.2528139 1.0763582-2.3283634 2.1538058-3.2266484 3.2323429-1.0744619 1.4358706-1.7911324 2.7976598-2.1500114 4.0853677s-.4632801 2.214814-.3132035 2.7813184c.1500767.5665043.1348515 1.1798542-.0456755 1.8400496 3.1059346-.1786668 5.8236273.5545206 8.1530782 2.199562s3.9726817 3.7095139 4.9296924 6.1934174c.9570106 2.4839036 1.2256261 5.2063926.8058464 8.1674672-.4197796 2.9610745-1.6443183 5.6094822-3.6736158 7.9452231-3.3451873 3.6495952-7.5712593 5.2205552-12.6782161 4.7128802s-8.9154242-2.6767329-11.4254021-6.5071737c-1.4942416-2.1548953-2.3609888-4.8479697-2.6002414-8.0792232-.2392527-3.2312535.0598131-6.1040841.8971975-8.6184917z"/>
  </svg>
`;function Qt(e,t){return e>=1e6?!0:t===`short`&&e>=1e3}function $t(e,t=`long`,n=`short`,r=M()){let i=e??-1;return i<0?``:Qt(i,t)?new Intl.NumberFormat(r,{notation:`compact`,compactDisplay:n===`short`?`short`:`long`}).format(i):new Intl.NumberFormat(r).format(i)}var R=class extends c{constructor(){super(...arguments),this.showTvClips=!1}render(){return a`
      <div class="item-stats">
        <p class="sr-only">
          ${this.mediatype===`account`?o(`Account Stats`):o(`Item Stats`)}
        </p>
        <ul id="stats-row">
          ${this.mediatypeIconColumnTemplate}
          ${this.mediatype===`account`?this.uploadsColumnTemplate:this.viewsColumnTemplate}
          ${this.favoritesColumnTemplate}
          ${this.showTvClips?this.tvClipsColumnTemplate:this.reviewsColumnTemplate}
        </ul>
      </div>
    `}get mediatypeIconColumnTemplate(){return a`
      <li class="col">
        <p class="sr-only">${o(`Mediatype:`)}</p>
        <tile-mediatype-icon .model=${this.model}></tile-mediatype-icon>
      </li>
    `}columnTemplate(e,t,n,i=[]){let s=$t(e??0,`short`,`short`),c=o(r`${s} ${t}`),l=t+`:`;return a`
      <li class="col ${i.join(` `)}" title=${c}>
        ${n}
        <p class="status-text">
          <span class="sr-only">${l}</span>
          ${s}
        </p>
      </li>
    `}get viewsColumnTemplate(){let e=this.viewLabel??o(`all-time views`);return this.columnTemplate(this.viewCount,e,Xt)}get uploadsColumnTemplate(){return this.columnTemplate(this.itemCount,o(`uploads`),Yt)}get favoritesColumnTemplate(){return this.columnTemplate(this.favCount,o(`favorites`),vt)}get reviewsColumnTemplate(){return this.columnTemplate(this.commentCount,o(`reviews`),Jt,[`reviews`])}get tvClipsColumnTemplate(){return this.columnTemplate(this.tvClipCount,o(`clips`),Zt)}static get styles(){return[j,e`
        tile-mediatype-icon {
          --iconHeight: 25px;
          --iconWidth: 25px;
        }

        ul {
          all: unset;
          list-style-type: none;
        }

        li {
          list-style-type: none;
        }

        svg {
          height: 13px;
          width: 13px;
          display: block;
          margin: auto;
          pointer-events: none;
        }

        /* Make the reviews icon slightly smaller/lower, for even visual weight */
        .reviews svg {
          height: 11px;
          width: 11px;
          margin-top: 2px;
        }

        .item-stats {
          height: 30px;
          padding-left: 5px;
          padding-right: 5px;
          font-family: 'Helvetica Neue', ui-sans-serif, system-ui, sans-serif;
          text-align: center;
        }

        #stats-row {
          display: flex;
          justify-content: space-between;
          flex-wrap: wrap;
          width: 100%;
          padding-bottom: 5px;
        }

        .col {
          min-width: 15px;
          max-width: 25%;
          height: 25px;
        }

        .status-text {
          font-size: 14px;
          height: 15px;
          color: #2c2c2c;
          line-height: 17px;
          margin: auto;
          display: block;
          text-align: center;
        }
      `]}};C([t({type:Object})],R.prototype,`model`,void 0),C([t({type:String})],R.prototype,`mediatype`,void 0),C([t({type:Number})],R.prototype,`itemCount`,void 0),C([t({type:Number})],R.prototype,`viewCount`,void 0),C([t({type:String})],R.prototype,`viewLabel`,void 0),C([t({type:Number})],R.prototype,`favCount`,void 0),C([t({type:Number})],R.prototype,`commentCount`,void 0),C([t({type:Boolean})],R.prototype,`showTvClips`,void 0),C([t({type:Number})],R.prototype,`tvClipCount`,void 0),R=C([x(`tile-stats`),i()],R);var en=class extends F{constructor(){super(...arguments),this.showInfoButton=!1,this.showTvClips=!1,this.layoutType=`default`}render(){let e=this.model?.title;return a`
      <div class=${g({container:!0,simple:this.layoutType!=="default","stats-only":this.layoutType===`stats-only`,"snippets-only":this.layoutType===`snippets-only`,minimal:this.layoutType===`minimal`})}>
        ${this.infoButtonTemplate}
        <div class="tile-details">
          <div class="item-info">
            ${this.imageBlockTemplate}

            <div id="title">
              <h3 class="truncated" title=${b(e)}>
                ${e}
              </h3>
            </div>

            ${this.volumeIssueTemplate}
            ${this.isSortedByDate?this.sortedDateInfoTemplate:this.creatorTemplate}
            ${this.webArchivesCaptureDatesTemplate} ${this.textSnippetsTemplate}
            ${this.reviewBlockTemplate}
          </div>

          ${this.tileStatsTemplate}
        </div>
      </div>
    `}get creatorTemplate(){let e=this.displayValueProvider.firstCreatorMatchingFilter;return e?a`
      <div class="created-by">
        <span class="truncated" title=${e}>
          ${o(r`by\u00a0${e}`)}
        </span>
      </div>
    `:n}get imageBlockTemplate(){return a`
      <image-block
        .model=${this.model}
        .baseImageUrl=${this.baseImageUrl}
        .loggedIn=${this.loggedIn}
        .suppressBlurring=${this.suppressBlurring}
        .isCompactTile=${!1}
        .isListTile=${!1}
        .viewSize=${`grid`}
      >
      </image-block>
    `}get sortedDateInfoTemplate(){let e,t=`long`;switch(this.effectiveSort?.field){case`date`:{let n=this.model?.datePublished;e={field:`published`,value:n},_t(n)&&(t=`year-only`);break}case`reviewdate`:e={field:`reviewed`,value:this.model?.dateReviewed};break;case`addeddate`:e={field:`added`,value:this.model?.dateAdded};break;case`publicdate`:e={field:`archived`,value:this.model?.dateArchived};break;default:break}if(!e?.value)return n;let r=this.getFormattedDate(e.value,t);return a`
      <div class="date-sorted-by truncated">
        <span>${e.field} ${r}</span>
      </div>
    `}get infoButtonTemplate(){return this.showInfoButton?a`<button class="info-button" @click=${this.infoButtonPressed}>
          &#9432;
          <span class="sr-only">${o(`More info`)}</span>
        </button>`:n}get reviewBlockTemplate(){if(!this.model?.review)return n;let{reviewtitle:e,reviewbody:t,stars:r}=this.model.review;return a`
      <review-block
        viewsize="grid"
        title=${b(e)}
        body=${b(t)}
        starRating=${b(r)}
      >
      </review-block>
    `}get textSnippetsTemplate(){return!this.hasSnippets||[`stats-only`,`minimal`].includes(this.layoutType)?n:a`
      <text-snippet-block viewsize="grid" .snippets=${this.model?.snippets}>
      </text-snippet-block>
    `}get volumeIssueTemplate(){return!this.model?.volume||!this.model?.issue?n:a`
      <div class="volume-issue">
        <span class="truncated" title=${o(`volume|issue`)}>
          ${o(r`Volume\u00a0${this.model.volume}, Issue\u00a0${this.model.issue}`)}
        </span>
      </div>
    `}get webArchivesCaptureDatesTemplate(){return!this.model?.captureDates||!this.model.title?n:a`
      <ul class="capture-dates">
        ${y(this.model.captureDates,e=>a`<li>
              ${this.displayValueProvider.webArchivesCaptureLink(this.model.title,e)}
            </li>`)}
      </ul>
    `}get tileStatsTemplate(){if([`snippets-only`,`minimal`].includes(this.layoutType))return n;let[e,t]=(this.sortParam??this.defaultSortParam)?.field===`week`?[this.model?.weeklyViewCount,o(`weekly views`)]:[this.model?.viewCount,o(`all-time views`)];return a`
      <tile-stats
        .model=${this.model}
        .mediatype=${this.model?.mediatype}
        .viewCount=${e}
        .viewLabel=${t}
        .favCount=${this.model?.favCount}
        .commentCount=${this.model?.commentCount}
        .tvClipCount=${this.model?.tvClipCount}
        .showTvClips=${this.showTvClips}
      >
      </tile-stats>
    `}get isSortedByDate(){return[`date`,`reviewdate`,`addeddate`,`publicdate`].includes(this.effectiveSort?.field)}get effectiveSort(){return this.sortParam??this.defaultSortParam}get hasSnippets(){return!!this.model?.snippets?.length}infoButtonPressed(e){e.preventDefault();let t=new CustomEvent(`infoButtonPressed`,{detail:{x:e.clientX,y:e.clientY}});this.dispatchEvent(t)}static get styles(){return[dt,e`
        a:link {
          text-decoration: none;
          color: var(--ia-theme-link-color, #4b64ff);
        }
        a:hover {
          text-decoration: underline;
        }

        .container {
          border: 1px solid ${e`var(--tileBorderColor, #dddddd)`};
        }

        .simple #title > .truncated {
          -webkit-line-clamp: 2;
        }

        .simple .created-by > .truncated,
        .simple .date-sorted-by > .truncated,
        .simple .volume-issue > .truncated {
          -webkit-line-clamp: 1;
        }

        .simple.snippets-only .item-info,
        .simple.minimal .item-info {
          padding-bottom: 5px;
        }

        .simple.snippets-only text-snippet-block {
          margin-top: auto; /* Force the snippets to the bottom of the tile */
        }

        .capture-dates {
          margin: 0;
          padding: 0 5px;
          list-style-type: none;
        }

        review-block,
        text-snippet-block {
          --containerLeftMargin: 5px;
          --containerTopMargin: 5px;
        }

        /**
         * iOS Safari long-press on tiles (to bring up hover pane)
         * gets messy without this
         */
        @media screen and (pointer: coarse) and (hover: none) {
          .container {
            -webkit-touch-callout: none;
          }

          .truncated {
            -webkit-touch-callout: default;
          }
        }
      `]}};C([t({type:Boolean})],en.prototype,`showInfoButton`,void 0),C([t({type:Boolean})],en.prototype,`showTvClips`,void 0),C([t({type:String})],en.prototype,`layoutType`,void 0),en=C([x(`item-tile`),i()],en);var tn=class extends F{constructor(){super(...arguments),this.showInfoButton=!1}render(){return a`
      <div class="container">
        ${this.infoButtonTemplate}
        <div class="tile-details">
          <div class="item-info">
            ${this.getAvatarTemplate} ${this.getTitleTemplate}
            ${this.getArchivistTemplate}
          </div>
          ${this.getTileStatsTemplate}
        </div>
      </div>
    `}get getAvatarTemplate(){return a`
      <image-block
        .model=${this.model}
        .baseImageUrl=${this.baseImageUrl}
        .viewSize=${`grid`}
        .suppressBlurring=${this.suppressBlurring}
      >
      </image-block>
    `}get getTitleTemplate(){return a`<div id="title">
      <h4 class="truncated">${this.model?.identifier}</h4>
    </div>`}get getArchivistTemplate(){return a`<div class="archivist-since">
      <span>${this.displayValueProvider.accountLabel}</span>
    </div>`}get getTileStatsTemplate(){return a`<tile-stats
      .mediatype=${this.model?.mediatype}
      .itemCount=${this.model?.itemCount}
      .favCount=${this.model?.favCount}
      .commentCount=${this.model?.commentCount}
    >
    </tile-stats>`}get infoButtonTemplate(){return this.showInfoButton?a`<button class="info-button" @click=${this.infoButtonPressed}>
          &#9432;
          <span class="sr-only">${o(`More info`)}</span>
        </button>`:n}infoButtonPressed(e){e.preventDefault();let t=new CustomEvent(`infoButtonPressed`,{detail:{x:e.clientX,y:e.clientY}});this.dispatchEvent(t)}static get styles(){return[dt,e`
        .container {
          border: 1px solid ${e`var(--tileBorderColor, #dddddd)`};
        }
      `]}};C([t({type:Boolean})],tn.prototype,`showInfoButton`,void 0),tn=C([x(`account-tile`),i()],tn);var nn=class extends F{constructor(){super(...arguments),this.showInfoButton=!1}render(){return a`
      <div class="container">
        <div class="tile-details">
          <div class="item-info">
            ${this.getImageBlockTemplate} ${this.getTitleTemplate}
          </div>
        </div>
      </div>
    `}get getImageBlockTemplate(){return a`
      <image-block
        .model=${this.model}
        .baseImageUrl=${this.baseImageUrl}
        .viewSize=${`grid`}
        .suppressBlurring=${this.suppressBlurring}
      >
      </image-block>
    `}get getTitleTemplate(){return a`<div id="title">
      <h3 class="truncated">${this.model?.title}</h3>
    </div>`}static get styles(){let t=e`var(--tileBorderColor, #555555)`;return[dt,e`
        .container {
          background-color: ${e`var(--tileBackgroundColor, #666666)`};
          border: 1px solid ${t};
        }

        .item-info {
          flex-grow: initial;
        }

        h4.truncated,
        h3.truncated {
          color: ${e`#fff`};
          -webkit-line-clamp: 4;
        }

        .container:hover > #title {
          text-decoration: underline;
        }

        /* this is a workaround for Safari 15 where the hover effects are not working */
        image-block:hover > #title {
          text-decoration: underline;
        }
      `]}};C([t({type:Boolean})],nn.prototype,`showInfoButton`,void 0),nn=C([x(`search-tile`)],nn);function rn(e,t,n,r=20,i=0){let a=[];if(i>=r)return a;let o=e=>{let a=e.assignedNodes().filter(e=>e.nodeType===1);return a.length>0?rn(a[0].parentElement,t,n,r,i+1):[]},s=Array.from(e.children||[]);for(let e of s)t(e)||(n(e)&&a.push(e),e.shadowRoot==null?e.tagName===`SLOT`?a.push(...o(e)):a.push(...rn(e,t,n,r,i+1)):a.push(...rn(e.shadowRoot,t,n,r,i+1)));return a}function an(e){return e.hasAttribute(`hidden`)||e.hasAttribute(`aria-hidden`)&&e.getAttribute(`aria-hidden`)!==`false`||e.style.display===`none`||e.style.opacity===`0`||e.style.visibility===`hidden`||e.style.visibility===`collapse`}function on(e){return e.hasAttribute(`disabled`)||e.hasAttribute(`aria-disabled`)&&e.getAttribute(`aria-disabled`)!==`false`}function sn(e){return e.getAttribute(`tabindex`)===`-1`||an(e)||on(e)?!1:e.hasAttribute(`tabindex`)||(e instanceof HTMLAnchorElement||e instanceof HTMLAreaElement)&&e.hasAttribute(`href`)||e instanceof HTMLButtonElement||e instanceof HTMLInputElement||e instanceof HTMLTextAreaElement||e instanceof HTMLSelectElement||e instanceof HTMLIFrameElement}var cn=new Map;function ln(e,t,n){let r=cn.get(n);r!=null&&window.clearTimeout(r),cn.set(n,window.setTimeout(()=>{e(),cn.delete(n)},t))}var un=document.createElement(`template`);un.innerHTML=`
	<div id="start"></div>
	<div id="backup"></div>
	<slot></slot>
	<div id="end"></div>
`;var dn=class extends HTMLElement{constructor(){super(),this.debounceId=Math.random().toString(),this._focused=!1;let e=this.attachShadow({mode:`open`});e.appendChild(un.content.cloneNode(!0)),this.$backup=e.querySelector(`#backup`),this.$start=e.querySelector(`#start`),this.$end=e.querySelector(`#end`),this.focusLastElement=this.focusLastElement.bind(this),this.focusFirstElement=this.focusFirstElement.bind(this),this.onFocusIn=this.onFocusIn.bind(this),this.onFocusOut=this.onFocusOut.bind(this)}static get observedAttributes(){return[`inactive`]}get inactive(){return this.hasAttribute(`inactive`)}set inactive(e){e?this.setAttribute(`inactive`,``):this.removeAttribute(`inactive`)}get focused(){return this._focused}connectedCallback(){this.$start.addEventListener(`focus`,this.focusLastElement),this.$end.addEventListener(`focus`,this.focusFirstElement),this.addEventListener(`focusin`,this.onFocusIn),this.addEventListener(`focusout`,this.onFocusOut),this.render()}disconnectedCallback(){this.$start.removeEventListener(`focus`,this.focusLastElement),this.$end.removeEventListener(`focus`,this.focusFirstElement),this.removeEventListener(`focusin`,this.onFocusIn),this.removeEventListener(`focusout`,this.onFocusOut)}attributeChangedCallback(){this.render()}focusFirstElement(){this.trapFocus()}focusLastElement(){this.trapFocus(!0)}getFocusableElements(){return rn(this,an,sn)}trapFocus(e){if(this.inactive)return;let t=this.getFocusableElements();t.length>0?(e?t[t.length-1].focus():t[0].focus(),this.$backup.setAttribute(`tabindex`,`-1`)):(this.$backup.setAttribute(`tabindex`,`0`),this.$backup.focus())}onFocusIn(){this.updateFocused(!0)}onFocusOut(){this.updateFocused(!1)}updateFocused(e){ln(()=>{this.focused!==e&&(this._focused=e,this.render())},0,this.debounceId)}render(){this.$start.setAttribute(`tabindex`,!this.focused||this.inactive?`-1`:`0`),this.$end.setAttribute(`tabindex`,!this.focused||this.inactive?`-1`:`0`),this.focused?this.setAttribute(`focused`,``):this.removeAttribute(`focused`)}};window.customElements.define(`focus-trap`,dn);var fn=e`
  .tile-actions {
    flex-shrink: 0;
    display: flex;
    gap: var(--tileActionGap, 0);
  }

  .tile-action-btn {
    flex: 1;
    padding: 6px 5px;
    border: 2px solid
      var(--tileActionBorderColor, var(--primaryErrorCTAFill, #d9534f));
    border-radius: var(--tileActionBorderRadius, 0);
    /* Inherit from the surrounding tile rather than the UA default for <button> */
    font-family: inherit;
    font-size: 1.2rem;
    font-weight: bold;
    cursor: pointer;
    color: var(--tileActionColor, var(--primaryErrorCTAFill, #d9534f));
    background: var(--tileActionBg, #fff);
    transition:
      background 0.15s,
      color 0.15s;
  }

  /*
   * When buttons are flush against each other (no gap), overlap their shared
   * edge by 1px so adjacent borders don't double up.
   */
  .tile-action-btn + .tile-action-btn {
    margin-left: -1px;
  }

  .tile-action-btn:hover {
    background: var(
      --tileActionHoverBg,
      rgba(var(--primaryErrorCTAFillRGB, 229, 28, 38), 0.2)
    );
    color: var(
      --tileActionHoverColor,
      var(--tileActionColor, var(--primaryErrorCTAFill, #d9534f))
    );
  }
`,pn=class extends F{constructor(){super(...arguments),this.collectionLinks=[]}render(){return a`
      <div id="list-line" class="${this.classSize}">
        ${this.classSize===`mobile`?this.mobileTemplate:this.desktopTemplate}
      </div>
    `}get mobileTemplate(){return a`
      <div id="list-line-top">
        <div id="list-line-left">
          ${this.imageBlockTemplate} ${this.renderTileActions(`list-detail`)}
        </div>
        <div id="list-line-right">
          <div id="title-line">
            <div id="title">${this.titleTemplate}</div>
            ${this.iconRightTemplate}
          </div>
        </div>
      </div>
      <div id="list-line-bottom">${this.detailsTemplate}</div>
    `}get desktopTemplate(){return a`
      <div id="list-line-left">
        ${this.imageBlockTemplate} ${this.renderTileActions(`list-detail`)}
      </div>
      <div id="list-line-right">
        <div id="title-line">
          <div id="title">${this.titleTemplate}</div>
          ${this.iconRightTemplate}
        </div>
        ${this.detailsTemplate}
      </div>
    `}get imageBlockTemplate(){if(!this.model)return n;let e=this.model.mediatype===`collection`,t=this.displayValueProvider.itemPageUrl(this.model.identifier,e);return a`<a
      id="image-link"
      title=${o(r`View ${this.model?.title}`)}
      href=${t}
    >
      <image-block
        .model=${this.model}
        .baseImageUrl=${this.baseImageUrl}
        .isCompactTile=${!1}
        .isListTile=${!0}
        .viewSize=${this.classSize}
        .loggedIn=${this.loggedIn}
        .suppressBlurring=${this.suppressBlurring}
      >
      </image-block>
    </a> `}get detailsTemplate(){return a`
      ${this.itemLineTemplate} ${this.creatorTemplate}
      <div id="dates-line">
        ${this.datePublishedTemplate} ${this.dateSortByTemplate}
        ${this.webArchivesCaptureDatesTemplate}
      </div>
      <div id="views-line">
        ${this.viewsTemplate} ${this.ratingTemplate} ${this.reviewsTemplate}
      </div>
      ${this.topicsTemplate} ${this.collectionsTemplate}
      ${this.descriptionTemplate} ${this.textSnippetsTemplate}
      ${this.reviewBlockTemplate}
    `}get iconRightTemplate(){return a`
      <a
        id="icon-right"
        href=${this.mediatypeURL}
        title=${o(r`See more: ${this.model?.mediatype}`)}
      >
        <tile-mediatype-icon .model=${this.model}> </tile-mediatype-icon>
      </a>
    `}get titleTemplate(){return this.model?.title?this.model?.href?a`<a href="${this.baseNavigationUrl}${this.model.href}"
          >${this.model.title??this.model.identifier}</a
        >`:this.detailsLink(this.model.identifier,this.model.title,this.model.mediatype===`collection`):n}get itemLineTemplate(){let e=this.sourceTemplate,t=this.volumeTemplate,r=this.issueTemplate;return!e&&!t&&!r?n:a` <div id="item-line">${e} ${t} ${r}</div> `}get sourceTemplate(){return this.model?.source?a`
      <div id="source" class="metadata">
        ${this.labelTemplate(o(`Source`))}
        ${this.searchLink(`source`,this.model.source)}
      </div>
    `:n}get volumeTemplate(){return this.metadataTemplate(this.model?.volume,o(`Volume`))}get issueTemplate(){return this.metadataTemplate(this.model?.issue,o(`Issue`))}get creatorTemplate(){return this.model?.mediatype===`account`?a`
        <div id="creator" class="metadata">
          <span class="label"
            >${this.displayValueProvider.accountLabel??n}</span
          >
        </div>
      `:!this.model?.creators||this.model.creators.length===0?n:a`
      <div id="creator" class="metadata">
        ${this.labelTemplate(o(`By`))}
        ${u(y(this.model.creators,e=>this.searchLink(`creator`,e)),`, `)}
      </div>
    `}get datePublishedTemplate(){let e=this.model?.datePublished,t=`long`;return _t(e)&&(t=`year-only`),this.metadataTemplate(this.getFormattedDate(e,t),o(`Published`))}get dateSortByTemplate(){return this.effectiveSort&&(this.effectiveSort.field===`addeddate`||this.effectiveSort.field===`reviewdate`||this.effectiveSort.field===`publicdate`)?this.metadataTemplate(this.getFormattedDate(this.date,`long`),this.displayValueProvider.dateLabel):n}get viewsTemplate(){let e=this.effectiveSort?.field===`week`?this.model?.weeklyViewCount:this.model?.viewCount;return e==null?n:this.model?.mediatype===`search`?this.metadataTemplate(o(`(Favorited search query)`),``):this.metadataTemplate(`${$t(e,this.formatSize)}`,this.displayValueProvider.viewsLabel)}get ratingTemplate(){let{averageRating:e}=this.model??{},t=e&&new Intl.NumberFormat(M()).format(e);return this.metadataTemplate(t,o(`Avg Rating`))}get reviewsTemplate(){let{commentCount:e}=this.model??{},t=e&&new Intl.NumberFormat(M()).format(e);return this.metadataTemplate(t,o(`Reviews`))}get topicsTemplate(){return!this.model?.subjects||this.model.subjects.length===0?n:a`
      <div id="topics" class="metadata">
        ${this.labelTemplate(o(`Topics`))}
        ${u(y(this.model.subjects,e=>this.searchLink(`subject`,e)),`, `)}
      </div>
    `}get collectionsTemplate(){return!this.collectionLinks||this.collectionLinks.length===0?n:a`
      <div id="collections" class="metadata">
        ${this.labelTemplate(o(`Collections`))}
        ${u(this.collectionLinks,`, `)}
      </div>
    `}get descriptionTemplate(){return this.metadataTemplate(h(le.sanitize(this.model?.description?.replace(/\n/g,` `)??``)),``,`description`)}get reviewBlockTemplate(){if(!this.model?.review)return n;let{reviewtitle:e,reviewbody:t,stars:r}=this.model.review;return a`
      <review-block
        viewsize="list"
        title=${b(e)}
        body=${b(t)}
        starRating=${b(r)}
      >
      </review-block>
    `}get textSnippetsTemplate(){return this.hasSnippets?a`<text-snippet-block
      viewsize="list"
      .snippets=${this.model?.snippets}
    ></text-snippet-block>`:n}get hasSnippets(){return!!this.model?.snippets?.length}get webArchivesCaptureDatesTemplate(){return!this.model?.captureDates||!this.model.title?n:a`
      <ul class="capture-dates">
        ${y(this.model.captureDates,e=>a`<li>
              ${this.displayValueProvider.webArchivesCaptureLink(this.model.title,e)}
            </li>`)}
      </ul>
    `}metadataTemplate(e,t=``,r){return e?a`
      <div id=${b(r)} class="metadata">
        ${this.labelTemplate(t)} ${e}
      </div>
    `:n}labelTemplate(e){return a` ${e?a`<span class="label">${e}: </span>`:n}`}searchLink(e,t){if(!e||!t)return n;let r=encodeURIComponent(`${e}:"${t}"`);return a`<a
      href="${this.baseNavigationUrl}/search?query=${r}"
      rel="nofollow"
    >
      ${le.sanitize(t)}</a
    >`}detailsLink(e,t,r=!1){if(!e)return n;let i=t??e;return a`<a href=${this.displayValueProvider.itemPageUrl(e,r)}> ${le.sanitize(i)} </a>`}get mediatypeURL(){if(this.baseNavigationUrl===void 0||!this.model?.mediatype)return n;switch(this.model.mediatype){case`collection`:return`${this.baseNavigationUrl}/search?query=mediatype:collection&sort=-downloads`;case`account`:return n;default:return this.displayValueProvider.itemPageUrl(this.model.mediatype,!0)}}updated(e){(e.has(`model`)||e.has(`collectionTitles`))&&this.buildCollectionLinks()}async buildCollectionLinks(){if(!this.model?.collections||this.model.collections.length===0)return;this.collectionLinks=[];let e=[];for(let t of this.model.collections)!Ne[t]&&!t.startsWith(`fav-`)&&e.push(this.detailsLink(t,this.collectionTitles?.get(t)??t,!0));this.collectionLinks=e}get date(){switch(this.effectiveSort?.field){case`date`:return this.model?.datePublished;case`reviewdate`:return this.model?.dateReviewed;case`addeddate`:return this.model?.dateAdded;default:return this.model?.dateArchived}}get effectiveSort(){return this.sortParam??this.defaultSortParam}get classSize(){return this.mobileBreakpoint&&this.currentWidth&&this.currentWidth<this.mobileBreakpoint?`mobile`:`desktop`}get formatSize(){return this.mobileBreakpoint&&this.currentWidth&&this.currentWidth<this.mobileBreakpoint?`short`:`long`}static get styles(){return[fn,e`
        html {
          font-size: unset;
        }

        div {
          font-size: 14px;
        }

        div a {
          text-decoration: none;
        }

        div a:link {
          color: var(--ia-theme-link-color, #4b64ff);
        }

        .label {
          font-weight: bold;
        }

        #list-line.mobile {
          --infiniteScrollerRowGap: 20px;
          --infiniteScrollerRowHeight: auto;
        }

        #list-line.desktop {
          --infiniteScrollerRowGap: 30px;
          --infiniteScrollerRowHeight: auto;
        }

        /* fields */
        #icon-right {
          width: 20px;
          padding-top: 5px;
          --iconHeight: 20px;
          --iconWidth: 20px;
          --iconTextAlign: right;
          margin-top: -8px;
          text-align: right;
        }

        #title {
          color: #4b64ff;
          text-decoration: none;
          font-size: 22px;
          font-weight: bold;
          /* align top of text with image */
          line-height: 25px;
          margin-top: -4px;
          padding-bottom: 2px;
          flex-grow: 1;

          display: -webkit-box;
          -webkit-box-orient: vertical;
          -webkit-line-clamp: 3;
          overflow: hidden;
          overflow-wrap: anywhere;
        }

        .metadata {
          line-height: 20px;
        }

        #description,
        #creator,
        #topics,
        #source {
          text-align: left;
          overflow: hidden;
          text-overflow: ellipsis;
          -webkit-box-orient: vertical;
          display: -webkit-box;
          word-break: break-word;
          -webkit-line-clamp: 3; /* number of lines to show */
          line-clamp: 3;

          /*
         * Safari doesn't always respect the line-clamping rules above,
         * so we add this to ensure these fields still get truncated
         */
          max-height: 60px;
        }

        #collections {
          display: -webkit-box;
          -webkit-box-orient: vertical;
          -webkit-line-clamp: 3;
          overflow: hidden;
          overflow-wrap: anywhere;
        }

        #collections > a {
          display: inline-block;
        }

        #icon {
          padding-top: 5px;
        }

        #description {
          padding-top: 10px;
        }

        /* Top level container */
        #list-line {
          display: flex;
        }

        #list-line.mobile {
          flex-direction: column;
        }

        #list-line.desktop {
          column-gap: 10px;
        }

        #list-line-top {
          display: flex;
          column-gap: 7px;
        }

        #list-line-bottom {
          padding-top: 4px;
        }

        #list-line-right,
        #list-line-top,
        #list-line-bottom {
          width: 100%;
        }

        /*
       * If the container becomes very tiny, don't let the thumbnail side take
       * up too much space. Shouldn't make a difference on ordinary viewport sizes.
       */
        #list-line-left {
          max-width: 25%;

          display: flex;
          flex-direction: column;
          row-gap: 5px;
        }

        div a:hover {
          text-decoration: underline;
        }

        /* Lines containing multiple div as row */
        #item-line,
        #dates-line,
        #views-line,
        #title-line {
          display: flex;
          flex-direction: row;
          column-gap: 10px;
        }

        /*
       * With the exception of the title line, allow these to wrap if
       * the space becomes too small to accommodate them together.
       *
       * The title line is excluded because it contains the mediatype icon
       * which we don't want to wrap.
       */
        #item-line,
        #dates-line,
        #views-line {
          flex-wrap: wrap;
        }

        .capture-dates {
          margin: 0;
          padding: 0;
          list-style-type: none;
        }

        .capture-dates a:link {
          text-decoration: none;
          color: var(--ia-theme-link-color, #4b64ff);
        }
        .capture-dates a:hover {
          text-decoration: underline;
        }

        /*
       * Tile-list (extended) action row sits under the thumbnail in the left
       * column. The image-block above already has its own bottom padding via
       * the parent's row-gap, so we don't need extra spacing here.
       */
        .tile-actions.list-detail {
          width: 100%;
        }
      `]}};C([t({type:Object})],pn.prototype,`collectionTitles`,void 0),C([s()],pn.prototype,`collectionLinks`,void 0),pn=C([x(`tile-list`),i()],pn);var z=class extends c{constructor(){super(...arguments),this.loggedIn=!1,this.suppressBlurring=!1}render(){return a`
      <div id="container">
        <focus-trap>
          ${this.headerTemplate}
          <div id="hover-tile-list">
            <tile-list
              .model=${this.model}
              .baseNavigationUrl=${this.baseNavigationUrl}
              .baseImageUrl=${this.baseImageUrl}
              .loggedIn=${this.loggedIn}
              .suppressBlurring=${this.suppressBlurring}
              .sortParam=${this.sortParam}
              .collectionTitles=${this.collectionTitles}
              .mobileBreakpoint=${this.mobileBreakpoint}
              .currentWidth=${this.currentWidth}
            ></tile-list>
          </div>
        </focus-trap>
      </div>
    `}get headerTemplate(){if(this.model?.collections?.length===0)return n;let e=``,t=``;for(let n of this.model?.collections||[])if(!Ne[n]&&!n.startsWith(`fav-`)){e=this.collectionTitles?.get(n)??n,t=n;break}return t?a`
      <div id="list-line-header">
        <a href="${this.baseNavigationUrl}/details/${t}">
          <img
            src="${this.baseImageUrl}/services/img/${t}"
            alt=""
          /><span>${e}</span>
        </a>
      </div>
    `:n}static get styles(){return e`
      :host {
        margin: 0;
        border: 0;
        padding: 0;
        overflow: visible;
        color: inherit;
        background: none;
        visibility: hidden;
        opacity: 0;
        transform: translateY(8px);
        transition:
          opacity 0.1s ease-in,
          transform 0.1s ease-in;
        --image-width: auto;
      }

      :host(.visible) {
        visibility: visible;
      }

      :host(.fade-in) {
        opacity: 1;
        transform: translateY(0);
      }

      @media (prefers-reduced-motion: reduce) {
        :host {
          transition-duration: 0.001s !important; /* Imperceptibly fast */
        }
      }

      #container {
        width: max-content;
        max-width: min(45vw, 600px);
        border: 1px solid #ddd;
        border-radius: 4px;
        box-shadow: 4px 4px 8px 0 rgba(0, 0, 0, 0.8);
        background: white;
      }

      @media screen and (max-width: 600px) {
        #container {
          max-width: 80vw;
        }
      }

      /* main tile-list container */
      #hover-tile-list {
        padding: 10px;
      }

      /* header on hover panel to show collection icon and title */
      #list-line-header {
        background: ${e`var(--hoverPaneHeaderBGColor, #edf0ff)`};
      }
      #list-line-header a {
        display: flex;
        align-items: center;
        column-gap: 5px;
        height: 3.4rem;
        padding: 0 10px;
        border-radius: 4px 4px 0 0;
        width: fit-content;
        font-size: 1.4rem;
        color: ${e`var(--ia-theme-link-color, #4b64ff)`};
        font-family: ${e`var(--ia-theme-base-font-family, "Helvetica Neue", Helvetica, Arial, sans-serif);`};
        text-decoration: none;
        width: auto;
      }
      #list-line-header a span {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
      }
      #list-line-header a:hover {
        text-decoration: underline;
      }
      #list-line-header a img {
        width: 30px;
        max-height: 30px;
      }
    `}};C([t({type:Object})],z.prototype,`model`,void 0),C([t({type:String})],z.prototype,`baseNavigationUrl`,void 0),C([t({type:String})],z.prototype,`baseImageUrl`,void 0),C([t({type:Boolean})],z.prototype,`loggedIn`,void 0),C([t({type:Boolean})],z.prototype,`suppressBlurring`,void 0),C([t({type:Object})],z.prototype,`sortParam`,void 0),C([t({type:Number})],z.prototype,`mobileBreakpoint`,void 0),C([t({type:Number})],z.prototype,`currentWidth`,void 0),C([t({type:Object})],z.prototype,`collectionTitles`,void 0),z=C([x(`tile-hover-pane`)],z);var mn=class extends F{render(){let e=this.tileActions.length>0;return a`
      <div id="list-line" class=${g({mobile:this.classSize===`mobile`,desktop:this.classSize===`desktop`,"has-actions":e})}>
        <image-block
          .model=${this.model}
          .baseImageUrl=${this.baseImageUrl}
          .isCompactTile=${!0}
          .isListTile=${!0}
          .viewSize=${this.classSize}
          .loggedIn=${this.loggedIn}
          .suppressBlurring=${this.suppressBlurring}
        >
        </image-block>
        ${e?a`<div id="actions">
              ${this.renderTileActions(`list-compact`)}
            </div>`:n}
        <a href=${this.href} id="title"
          >${le.sanitize(this.model?.title??``)}</a
        >
        <div id="creator">
          ${this.model?.mediatype===`account`?this.displayValueProvider.accountLabel:this.creator}
        </div>
        <div id="date">
          ${this.getFormattedDate(this.date,this.dateFormatSize)}
        </div>
        <div id="icon">
          <tile-mediatype-icon .model=${this.model}> </tile-mediatype-icon>
        </div>
        <div id="views">${$t(this.views??0,this.formatSize)}</div>
      </div>
    `}get href(){return!this.model?.identifier||this.baseNavigationUrl==null?n:this.model.href?`${this.baseNavigationUrl}${this.model.href}`:this.displayValueProvider.itemPageUrl(this.model.identifier,this.model.mediatype===`collection`)}get creator(){return this.displayValueProvider.firstCreatorMatchingFilter??n}get date(){switch(this.effectiveSort?.field){case`publicdate`:return this.model?.dateArchived;case`reviewdate`:return this.model?.dateReviewed;case`addeddate`:return this.model?.dateAdded;default:return this.model?.datePublished}}get views(){return this.effectiveSort?.field===`week`?this.model?.weeklyViewCount:this.model?.viewCount}get effectiveSort(){return this.sortParam??this.defaultSortParam}get classSize(){return this.mobileBreakpoint&&this.currentWidth&&this.currentWidth<this.mobileBreakpoint?`mobile`:`desktop`}get dateFormatSize(){return(!this.isSortedByDate||this.effectiveSort?.field===`date`)&&_t(this.model?.datePublished)?`year-only`:this.formatSize}get formatSize(){return this.mobileBreakpoint&&this.currentWidth&&this.currentWidth<this.mobileBreakpoint?`short`:`long`}get isSortedByDate(){return[`date`,`reviewdate`,`addeddate`,`publicdate`].includes(this.effectiveSort?.field)}static get styles(){return[fn,e`
        html {
          font-size: unset;
        }

        div {
          font-size: 14px;
        }

        #list-line {
          display: grid;
          column-gap: 10px;
          border-top: 1px solid #ddd;
          align-items: center;
          line-height: 20px;
          padding-top: 5px;
          margin-bottom: -5px;
        }

        #list-line.mobile {
          grid-template-columns: 36px 3fr 2fr 68px 35px;
        }

        #list-line.desktop {
          grid-template-columns: 51px 3fr 2fr 95px 30px 115px;
        }

        /*
         * When tile actions are present, insert an extra column for them
         * between the thumbnail and the title.
         */
        #list-line.mobile.has-actions {
          grid-template-columns:
            36px var(--tileActionColumnWidth, 90px)
            3fr 2fr 68px 35px;
        }

        #list-line.desktop.has-actions {
          grid-template-columns:
            51px var(--tileActionColumnWidth, 100px)
            3fr 2fr 95px 30px 115px;
        }

        #actions {
          /* The flex container inside is what holds the action buttons */
          display: flex;
        }

        #list-line:hover #title {
          text-decoration: underline;
        }

        #title {
          text-decoration: none;
        }

        #title:link {
          color: var(--ia-theme-link-color, #4b64ff);
        }

        #title,
        #creator {
          text-overflow: ellipsis;
          overflow: hidden;
          white-space: nowrap;
        }

        #icon {
          margin-left: 2px;
        }

        #views {
          text-align: right;
          padding-right: 8px;
        }

        .mobile #views {
          display: none;
        }

        .mobile tile-mediatype-icon {
          --iconHeight: 14px;
          --iconWidth: 14px;
        }

        .desktop #icon {
          --iconHeight: 20px;
          --iconWidth: 20px;
        }

        item-image {
          --imgHeight: 100%;
          --imgWidth: 100%;
        }
      `]}};mn=C([x(`tile-list-compact`),i()],mn);var hn=class extends F{render(){let e=this.tileActions.length>0;return a`
      <div id="list-line-header" class=${g({mobile:this.classSize===`mobile`,desktop:this.classSize===`desktop`,"has-actions":e})}>
        <div id="thumb"></div>
        ${e?a`<div id="actions-header"></div>`:n}
        <div id="title">${o(`Title`)}</div>
        <div id="creator">${o(`Creator`)}</div>
        <div id="date">
          ${this.displayValueProvider.dateLabel||o(`Published`)}
        </div>
        <div id="icon">${o(`Type`)}</div>
        <div id="views">${this.displayValueProvider.viewsLabel}</div>
      </div>
    `}get classSize(){return this.mobileBreakpoint&&this.currentWidth&&this.currentWidth<this.mobileBreakpoint?`mobile`:`desktop`}static get styles(){return e`
      html {
        font-size: unset;
      }

      div {
        font-size: 14px;
        font-weight: bold;
        line-height: 20px;
      }

      .mobile #views {
        display: none;
      }

      #views {
        text-align: right;
        padding-right: 8px;
      }

      #list-line-header {
        display: grid;
        column-gap: 10px;
        align-items: flex-end;
        padding-bottom: 2px;
      }

      #list-line-header.mobile {
        grid-template-columns: 36px 3fr 2fr 68px 35px;
      }

      #list-line-header.desktop {
        grid-template-columns: 51px 3fr 2fr 95px 30px 115px;
      }

      /*
       * When tile actions are present in the rows below, reserve a matching
       * column here so the columns stay aligned with each row.
       */
      #list-line-header.mobile.has-actions {
        grid-template-columns:
          36px var(--tileActionColumnWidth, 90px) 3fr 2fr
          68px 35px;
      }

      #list-line-header.desktop.has-actions {
        grid-template-columns:
          51px var(--tileActionColumnWidth, 100px) 3fr 2fr
          95px 30px 115px;
      }
    `}};hn=C([x(`tile-list-compact-header`),i()],hn);var gn=(e,t=-1/0,n=1/0)=>Math.max(t,Math.min(e,n)),_n=class{constructor(e,t={}){this.host=e,this.mobileBreakpoint=600,this.offsetX=-10,this.offsetY=15,this.showDelay=300,this.hideDelay=100,this.longPressDelay=600,this.enableLongPress=!1,this.hoverPaneState=`hidden`,this.forceTouchBackdrop=!1,this.lastPointerClientPos={x:0,y:0},this.clicking=!1,this.handleFocus=()=>{!this.clicking&&this.hoverPaneState===`hidden`&&this.showHoverPane({anchor:`host`}),this.clicking=!1},this.handleBlur=()=>{this.hoverPaneState!==`hidden`&&this.fadeOutHoverPane()},this.handlePointerDown=()=>{this.clicking=!0},this.handleKeyDown=e=>{(e.key===`ArrowDown`||e.key===`ArrowUp`)&&this.hoverPaneState!==`hidden`&&e.preventDefault()},this.handleKeyUp=e=>{if(this.hoverPaneState===`hidden`||!this.hoverPane)return;e.key===`ArrowDown`&&(this.hoverPane.tabIndex=1,this.hoverPane.focus());let t=e.key===`ArrowUp`,n=e.key===`Escape`||e.key===`Esc`;n&&this.fadeOutHoverPane(),(t||n)&&(this.hoverPane.tabIndex=-1,this.host.acquireFocus())},this.handleMouseEnter=e=>{this.handleMouseMove(e)},this.handleMouseMove=e=>{clearTimeout(this.hideTimer),this.hoverPaneState===`fading-out`&&(this.hoverPaneState=`shown`,this.hoverPane?.classList.add(`fade-in`)),this.hoverPaneState===`hidden`&&(this.restartShowHoverPaneTimer(),this.lastPointerClientPos={x:e.clientX,y:e.clientY})},this.handleMouseLeave=()=>{this.host.releaseFocus(),clearTimeout(this.showTimer),this.hoverPaneState!==`hidden`&&(this.hideTimer=window.setTimeout(()=>{this.fadeOutHoverPane()},this.hideDelay))},this.handleTouchStart=e=>{clearTimeout(this.longPressTimer),e.touches.length===1&&(this.longPressTimer=window.setTimeout(()=>{this.hoverPaneState===`hidden`&&this.showHoverPane()},this.longPressDelay),this.lastPointerClientPos={x:e.touches[0].clientX,y:e.touches[0].clientY})},this.handleLongPressCancel=()=>{clearTimeout(this.longPressTimer)},this.handleContextMenu=e=>{e.preventDefault()},this.handleBackdropInteraction=e=>{this.hoverPaneState!==`hidden`&&this.fadeOutHoverPane(),e.preventDefault(),e.stopPropagation()},this.mobileBreakpoint=t.mobileBreakpoint??this.mobileBreakpoint,this.offsetX=t.offsetX??this.offsetX,this.offsetY=t.offsetY??this.offsetY,this.showDelay=t.showDelay??this.showDelay,this.hideDelay=t.hideDelay??this.hideDelay,this.longPressDelay=t.longPressDelay??this.longPressDelay,this.enableLongPress=t.enableLongPress??this.enableLongPress,this.host.addController(this)}hostConnected(){this.attachListeners()}hostDisconnected(){this.detachListeners()}hostUpdated(){this.hoverPane=this.host.getHoverPane(),this.hoverPaneProps=this.host.getHoverPaneProps()}getTemplate(){return this.hoverPaneProps=this.host.getHoverPaneProps(),this.shouldRenderHoverPane?a`
          ${this.touchBackdropTemplate}
          <tile-hover-pane
            popover
            tabindex="-1"
            aria-describedby="tile-hover-pane-aria-description"
            .model=${this.hoverPaneProps?.model}
            .baseNavigationUrl=${this.hoverPaneProps?.baseNavigationUrl}
            .baseImageUrl=${this.hoverPaneProps?.baseImageUrl}
            .loggedIn=${this.hoverPaneProps?.loggedIn}
            .suppressBlurring=${this.hoverPaneProps?.suppressBlurring}
            .sortParam=${this.hoverPaneProps?.sortParam}
            .collectionTitles=${this.hoverPaneProps?.collectionTitles}
            .mobileBreakpoint=${this.mobileBreakpoint}
            .currentWidth=${window.innerWidth}
          ></tile-hover-pane>
          <div id="tile-hover-pane-aria-description" class="sr-only">
            ${o(`Press Up Arrow to exit item detail preview`)}
          </div>
        `:n}toggleHoverPane(e){this.hoverPaneState===`hidden`?(this.lastPointerClientPos=e.coords,this.forceTouchBackdrop=e.enableTouchBackdrop??!1,this.showHoverPane()):(this.fadeOutHoverPane(),this.forceTouchBackdrop=!1)}get touchBackdropTemplate(){return this.showTouchBackdrop?a`<div
          id="touch-backdrop"
          @touchstart=${this.handleBackdropInteraction}
          @touchmove=${this.handleBackdropInteraction}
          @touchend=${this.handleBackdropInteraction}
          @touchcancel=${this.handleBackdropInteraction}
          @mouseenter=${e=>e.stopPropagation()}
          @mousemove=${e=>e.stopPropagation()}
          @mouseleave=${e=>e.stopPropagation()}
        ></div>`:n}get showTouchBackdrop(){return this.isTouchEnabled&&this.enableLongPress||this.forceTouchBackdrop}get isMobileView(){return!!this.mobileBreakpoint&&window.innerWidth<this.mobileBreakpoint}get isHoverEnabled(){return window.matchMedia(`(hover: hover)`).matches}get isTouchEnabled(){return`ontouchstart`in window&&window.matchMedia(`(any-pointer: coarse)`).matches}get shouldRenderHoverPane(){return this.hoverPaneState!==`hidden`}makePaneDesiredOffsets(e){let[t,n]=[0,0];switch(e){case`host`:let e=this.host.getBoundingClientRect();t=e.left+20,n=e.top+30;break;case`cursor`:t=this.lastPointerClientPos.x,n=this.lastPointerClientPos.y;break}let r=t>window.innerWidth/2,i=n>window.innerHeight/2,a=this.hoverPane?.getBoundingClientRect();return a&&(r&&(t-=a.width),i&&(n-=a.height),t+=(r?-1:1)*this.offsetX,n+=(i?-1:1)*this.offsetY,this.isMobileView&&(t=gn(t,20,window.innerWidth-a.width-20),n=gn(n,20,window.innerHeight-a.height-20))),t+=window.scrollX,n+=window.scrollY,{left:t,top:n}}attachListeners(){this.host.addEventListener(`focus`,this.handleFocus),this.host.addEventListener(`blur`,this.handleBlur),this.host.addEventListener(`pointerdown`,this.handlePointerDown),this.host.addEventListener(`keyup`,this.handleKeyUp),this.host.addEventListener(`keydown`,this.handleKeyDown),this.isHoverEnabled&&(this.host.addEventListener(`mouseenter`,this.handleMouseEnter),this.host.addEventListener(`mousemove`,this.handleMouseMove),this.host.addEventListener(`mouseleave`,this.handleMouseLeave)),this.isTouchEnabled&&this.enableLongPress&&(this.host.addEventListener(`touchstart`,this.handleTouchStart),this.host.addEventListener(`touchmove`,this.handleLongPressCancel),this.host.addEventListener(`touchend`,this.handleLongPressCancel),this.host.addEventListener(`touchcancel`,this.handleLongPressCancel),this.host.addEventListener(`contextmenu`,this.handleContextMenu))}detachListeners(){this.host.removeEventListener(`mouseenter`,this.handleMouseEnter),this.host.removeEventListener(`mousemove`,this.handleMouseMove),this.host.removeEventListener(`mouseleave`,this.handleMouseLeave),this.host.removeEventListener(`touchstart`,this.handleTouchStart),this.host.removeEventListener(`touchmove`,this.handleLongPressCancel),this.host.removeEventListener(`touchend`,this.handleLongPressCancel),this.host.removeEventListener(`touchcancel`,this.handleLongPressCancel),this.host.removeEventListener(`contextmenu`,this.handleContextMenu),this.host.removeEventListener(`focus`,this.handleFocus),this.host.removeEventListener(`blur`,this.handleBlur),this.host.removeEventListener(`keyup`,this.handleKeyUp),this.host.removeEventListener(`keydown`,this.handleKeyDown)}restartShowHoverPaneTimer(){clearTimeout(this.showTimer),this.showTimer=window.setTimeout(()=>{this.host.acquireFocus(),this.showHoverPane()},this.showDelay)}async showHoverPane(e={anchor:`cursor`}){this.hoverPaneState=`shown`,this.host.requestUpdate(),await this.host.updateComplete,this.hoverPane?.isConnected&&(this.hoverPane&&!this.hoverPane.matches(`:popover-open`)&&this.hoverPane.showPopover?.(),await Promise.race([new Promise(e=>{requestAnimationFrame(e)}),new Promise(e=>{setTimeout(e,50)})]),this.repositionHoverPane(e.anchor),this.hoverPane?.classList.add(`visible`,`fade-in`))}fadeOutHoverPane(){this.hoverPaneState=`fading-out`,this.hoverPane?.classList.remove(`fade-in`),clearTimeout(this.hideTimer),this.hideTimer=window.setTimeout(()=>{this.hoverPaneState=`hidden`,this.hoverPane&&(this.hoverPane.tabIndex=-1),this.host.requestUpdate()},100)}repositionHoverPane(e){if(!this.hoverPane)return;let{top:t,left:n}=this.makePaneDesiredOffsets(e);this.hoverPane.style.top=`${t}px`,this.hoverPane.style.left=`${n}px`}},vn,yn,B=(vn=class extends F{constructor(){super(...arguments),this.isManageView=!1,this.showTvClips=!1,this.layoutType=`default`,this.enableHoverPane=!1,this.handleGridActionsMouseEnter=()=>{this.dispatchEvent(new MouseEvent(`mouseleave`,{bubbles:!1}))}}acquireFocus(){this.tileLinkElement?.focus()}releaseFocus(){this.tileLinkElement?.blur()}render(){let e=this.tileDisplayMode===`grid`,t=e&&this.showGridTileActions,r=this.hoverPaneController?.getTemplate()??n;return a`
      <div id="container" class=${g({hoverable:e,"has-tile-actions":t})}>
        ${this.tileDisplayMode===`list-header`?this.headerTemplate:this.tileTemplate}
        ${this.gridTileActionsTemplate} ${this.manageCheckTemplate}
        ${r}
      </div>
    `}firstUpdated(){this.shouldPrepareHoverPane&&(this.hoverPaneController=new _n(this,{mobileBreakpoint:this.mobileBreakpoint,enableLongPress:!1}))}get headerTemplate(){let{currentWidth:e,sortParam:t,defaultSortParam:n,mobileBreakpoint:r}=this;return a`
      <tile-list-compact-header
        class="header"
        .currentWidth=${e}
        .sortParam=${t??n}
        .mobileBreakpoint=${r}
        .tileActions=${this.tileActions}
      >
      </tile-list-compact-header>
    `}get tileTemplate(){return a`
      ${this.tileDisplayMode===`list-detail`?this.tile:this.linkTileTemplate}
    `}get linkTileTemplate(){return a`
      <a
        href=${this.linkTileHref}
        aria-label=${this.model?.title??o(`Untitled item`)}
        aria-describedby="link-aria-description"
        aria-haspopup=${this.shouldPrepareHoverPane?`dialog`:`false`}
        title=${this.shouldPrepareHoverPane?n:b(this.model?.title)}
        @click=${this.handleLinkClicked}
        @contextmenu=${this.handleLinkContextMenu}
        class="tile-link"
      >
        ${this.tile}
      </a>
      <div id="link-aria-description" class="sr-only">
        ${o(`Press Down Arrow to preview item details`)}
      </div>
    `}get linkTileHref(){return!this.model?.identifier||this.baseNavigationUrl==null?n:this.model.href?`${this.baseNavigationUrl}${this.model.href}`:this.displayValueProvider.itemPageUrl(this.model.identifier,this.model.mediatype===`collection`)}get manageCheckTemplate(){return!this.isManageView||this.tileDisplayMode!==`grid`?n:a`
      <div class="manage-check">
        <input
          type="checkbox"
          title=${this.manageCheckTitle??o(`Remove this item from the list`)}
          ?checked=${this.model?.checked}
          @change=${this.handleLinkClicked}
        />
      </div>
    `}get shouldPrepareHoverPane(){return this.enableHoverPane&&!!this.tileDisplayMode&&yn.HOVER_PANE_DISPLAY_MODES[this.tileDisplayMode]&&this.model?.mediatype!==`search`&&!this.model?.captureDates}get isHoverEnabled(){return window.matchMedia(`(hover: hover)`).matches}get shouldShowInfoButton(){return!this.isHoverEnabled&&this.shouldPrepareHoverPane}getHoverPane(){return this.hoverPane}getHoverPaneProps(){return this}handleResize(e){this.currentWidth=e.contentRect.width,this.currentHeight=e.contentRect.height}disconnectedCallback(){this.stopResizeObservation(this.resizeObserver)}stopResizeObservation(e){e?.removeObserver({handler:this,target:this.container})}startResizeObservation(){this.stopResizeObservation(this.resizeObserver),this.resizeObserver?.addObserver({handler:this,target:this.container})}updated(e){if(e.has(`resizeObserver`)){let t=e.get(`resizeObserver`);this.stopResizeObservation(t),this.startResizeObservation()}}handleLinkClicked(e){this.isManageView&&(e.preventDefault(),this.model&&(this.model.checked=!this.model.checked)),this.dispatchEvent(new CustomEvent(`resultSelected`,{detail:this.model}))}handleLinkContextMenu(e){this.isManageView&&this.linkTileHref!==n&&(e.preventDefault(),window.open(this.linkTileHref,`_blank`))}tileInfoButtonPressed(e){this.hoverPaneController?.toggleHoverPane({coords:e.detail,enableTouchBackdrop:!0})}get showGridTileActions(){return this.tileActions.length>0&&!this.isManageView&&this.tileDisplayMode===`grid`}get gridTileActionsTemplate(){return this.showGridTileActions?a`
      <div
        class="tile-actions grid-tile-actions"
        @mouseenter=${this.handleGridActionsMouseEnter}
        @mousemove=${e=>e.stopPropagation()}
      >
        ${this.tileActions.map(e=>a`
            <button
              class="tile-action-btn"
              @click=${t=>this.handleTileActionClick(t,e)}
            >
              ${e.label}
            </button>
          `)}
      </div>
    `:n}get tile(){let{model:e,collectionPagePath:t,baseNavigationUrl:r,currentWidth:i,currentHeight:o,sortParam:s,creatorFilter:c,mobileBreakpoint:l,defaultSortParam:u}=this;if(!e)return n;switch(this.tileDisplayMode){case`grid`:switch(e.mediatype){case`collection`:return a`<collection-tile
              .model=${e}
              .collectionPagePath=${t}
              .baseImageUrl=${this.baseImageUrl}
              .currentWidth=${i}
              .currentHeight=${o}
              .creatorFilter=${c}
              .suppressBlurring=${this.suppressBlurring}
              .isManageView=${this.isManageView}
              .layoutType=${this.layoutType}
              ?showInfoButton=${this.shouldShowInfoButton}
              @infoButtonPressed=${this.tileInfoButtonPressed}
            >
            </collection-tile>`;case`account`:return a`<account-tile
              .model=${e}
              .collectionPagePath=${t}
              .baseImageUrl=${this.baseImageUrl}
              .currentWidth=${i}
              .currentHeight=${o}
              .creatorFilter=${c}
              .suppressBlurring=${this.suppressBlurring}
              .isManageView=${this.isManageView}
              ?showInfoButton=${this.shouldShowInfoButton}
              @infoButtonPressed=${this.tileInfoButtonPressed}
            >
            </account-tile>`;case`search`:return a`<search-tile
              .model=${e}
              .collectionPagePath=${t}
              .baseImageUrl=${this.baseImageUrl}
              .currentWidth=${i}
              .currentHeight=${o}
              .creatorFilter=${c}
              .suppressBlurring=${this.suppressBlurring}
              .isManageView=${this.isManageView}
              ?showInfoButton=${!1}
              @infoButtonPressed=${this.tileInfoButtonPressed}
            >
            </search-tile>`;default:return a`<item-tile
              .model=${e}
              .collectionPagePath=${t}
              .currentWidth=${this.currentWidth}
              .currentHeight=${this.currentHeight}
              .baseImageUrl=${this.baseImageUrl}
              .sortParam=${s}
              .defaultSortParam=${u}
              .creatorFilter=${c}
              .loggedIn=${this.loggedIn}
              .suppressBlurring=${this.suppressBlurring}
              .isManageView=${this.isManageView}
              .layoutType=${this.layoutType}
              ?showTvClips=${this.showTvClips}
              ?showInfoButton=${this.shouldShowInfoButton}
              ?useLocalTime=${this.useLocalTime}
              @infoButtonPressed=${this.tileInfoButtonPressed}
            >
            </item-tile>`}case`list-compact`:return a`<tile-list-compact
          .model=${e}
          .collectionPagePath=${t}
          .currentWidth=${i}
          .currentHeight=${o}
          .baseNavigationUrl=${r}
          .sortParam=${s}
          .defaultSortParam=${u}
          .creatorFilter=${c}
          .mobileBreakpoint=${l}
          .baseImageUrl=${this.baseImageUrl}
          .loggedIn=${this.loggedIn}
          .suppressBlurring=${this.suppressBlurring}
          .tileActions=${this.isManageView?[]:this.tileActions}
          ?useLocalTime=${this.useLocalTime}
        >
        </tile-list-compact>`;case`list-detail`:return a`<tile-list
          .model=${e}
          .collectionPagePath=${t}
          .collectionTitles=${this.collectionTitles}
          .currentWidth=${i}
          .currentHeight=${o}
          .baseNavigationUrl=${r}
          .sortParam=${s}
          .defaultSortParam=${u}
          .creatorFilter=${c}
          .mobileBreakpoint=${l}
          .baseImageUrl=${this.baseImageUrl}
          .loggedIn=${this.loggedIn}
          .suppressBlurring=${this.suppressBlurring}
          .tileActions=${this.isManageView?[]:this.tileActions}
          ?useLocalTime=${this.useLocalTime}
        >
        </tile-list>`;default:return n}}static get styles(){return[j,fn,e`
        :host {
          display: block;
          height: 100%;
        }

        collection-tile {
          --tileBorderColor: #555555;
          --tileBackgroundColor: #666666;
          --imageBlockBackgroundColor: #666666;
        }

        account-tile {
          --tileBorderColor: #dddddd;
          --imageBlockBackgroundColor: #fcf5e6;
        }

        item-tile {
          --tileBorderColor: #dddddd;
          --imageBlockBackgroundColor: #f1f1f4;
        }

        search-tile {
          --tileBorderColor: #555555;
          --tileBackgroundColor: #666666;
          --imageBlockBackgroundColor: #666666;
          --iconFillColor: #2c2c2c;
        }

        #container {
          position: relative;
          height: 100%;
          border-radius: 4px;
        }

        /*
         * When tile actions are present, the container takes on the role of
         * the tile's visual card so the tile content and action row appear
         * as a single unified element. The inner tile's own shadow/radius
         * are disabled via CSS variable overrides to avoid visual
         * duplication, and the action row sits as a footer inside the same
         * card.
         */
        #container.has-tile-actions {
          display: flex;
          flex-direction: column;
          overflow: hidden;
          box-shadow: var(--tileShadow, 1px 1px 2px 0);
          --tileBoxShadow: none;
          --tileCornerRadius: 0;
        }

        #container.has-tile-actions .tile-link {
          flex: 1;
          min-height: 0;
          overflow: hidden;
          border-radius: 0;
        }

        /* Normal hover shadow lives on the inner anchor for plain tiles */
        #container.hoverable:not(.has-tile-actions) a:focus,
        #container.hoverable:not(.has-tile-actions) a:hover {
          box-shadow: var(
            --tileHoverBoxShadow,
            0 0 6px 2px rgba(8, 8, 32, 0.8)
          );
          transition: box-shadow 0.1s ease;
        }

        /*
         * When the container owns the card visuals, the hover shadow needs
         * to move up to the container so it wraps the action row too.
         */
        #container.hoverable.has-tile-actions:hover {
          box-shadow: var(
            --tileHoverBoxShadow,
            0 0 6px 2px rgba(8, 8, 32, 0.8)
          );
          transition: box-shadow 0.1s ease;
        }

        a {
          display: block;
          height: 100%;
          color: unset;
          text-decoration: none;
          transition: transform 0.05s ease;
          border-radius: 4px;
          outline: none;
        }

        a :first-child {
          display: block;
          height: 100%;
        }

        .manage-check {
          position: absolute;
          right: 0;
          top: 0;
          border: 5px solid #2c2c2c;
          border-radius: 3px;
          background-color: #2c2c2c;
          z-index: 1;
        }

        .manage-check > input[type='checkbox'] {
          display: block;
          margin: 0;
        }

        #touch-backdrop {
          position: fixed;
          width: 100vw;
          height: 100vh;
          top: 0;
          left: 0;
          z-index: 2;
          background: transparent;
        }

        tile-hover-pane {
          position: absolute;
          top: 0;
          left: -9999px;
          z-index: 2;
        }

        /*
         * Grid-mode action row sits flush against the bottom of the card —
         * the buttons' own borders form the visible bottom edge. The outer
         * buttons get rounded bottom corners to match the container so the
         * red border traces cleanly around the card's bottom corners.
         */
        .grid-tile-actions .tile-action-btn:first-child {
          border-bottom-left-radius: 4px;
        }

        .grid-tile-actions .tile-action-btn:last-child {
          border-bottom-right-radius: 4px;
        }
      `]}},yn=vn,vn.HOVER_PANE_DISPLAY_MODES={grid:!0,"list-compact":!0,"list-detail":!1,"list-header":!1},vn);C([t({type:String})],B.prototype,`tileDisplayMode`,void 0),C([t({type:Boolean})],B.prototype,`isManageView`,void 0),C([t({type:Object})],B.prototype,`resizeObserver`,void 0),C([t({type:Object})],B.prototype,`collectionTitles`,void 0),C([t({type:Boolean})],B.prototype,`showTvClips`,void 0),C([t({type:String})],B.prototype,`layoutType`,void 0),C([t({type:Boolean})],B.prototype,`enableHoverPane`,void 0),C([t({type:String})],B.prototype,`manageCheckTitle`,void 0),C([S(`#container`)],B.prototype,`container`,void 0),C([S(`tile-hover-pane`)],B.prototype,`hoverPane`,void 0),C([S(`.tile-link`)],B.prototype,`tileLinkElement`,void 0),B=yn=C([x(`tile-dispatcher`),i()],B);var bn=class extends c{render(){return a` <div id="container"></div> `}static get styles(){return e`
      :host {
        display: block;
        height: 100%;
      }

      #container {
        background: linear-gradient(to right bottom, #dddddd, #d8d8d8);
        border-radius: var(--tileCornerRadius, 4px);
        background-size: 100% 100%;
        display: block;
        height: 100%;
      }
    `}};bn=C([x(`collection-browser-loading-tile`)],bn);var xn=a`
  <svg viewBox="0 0 64 100" xmlns="http://www.w3.org/2000/svg">
    <g fill-rule="evenodd">
      <path
        d="m0 35.625 21.5-35.625 21.5 35.625h-14.3333333v21.375h-14.3333334v-21.375z"
        fill="#2c2c2c"
      />
      <path
        d="m21 78.625 21.5-35.625 21.5 35.625h-14.3333333v21.375h-14.3333334v-21.375z"
        fill="#bbb"
        transform="matrix(1 0 0 -1 0 143)"
      />
    </g>
  </svg>
`,Sn=a`
  <svg viewBox="0 0 64 100" xmlns="http://www.w3.org/2000/svg">
    <g fill-rule="evenodd">
      <path
        d="m0 35.625 21.5-35.625 21.5 35.625h-14.3333333v21.375h-14.3333334v-21.375z"
        fill="#bbb"
      />
      <path
        d="m21 78.625 21.5-35.625 21.5 35.625h-14.3333333v21.375h-14.3333334v-21.375z"
        fill="#2c2c2c"
        transform="matrix(1 0 0 -1 0 143)"
      />
    </g>
  </svg>
`,Cn=a`
  <svg viewBox="0 0 64 100" xmlns="http://www.w3.org/2000/svg">
    <g fill="#bbb" fill-rule="evenodd">
      <path
        d="m0 35.625 21.5-35.625 21.5 35.625h-14.3333333v21.375h-14.3333334v-21.375z"
      />
      <path
        d="m21 78.625 21.5-35.625 21.5 35.625h-14.3333333v21.375h-14.3333334v-21.375z"
        transform="matrix(1 0 0 -1 0 143)"
      />
    </g>
  </svg>
`,wn=m`
<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><path d="m64 54v46h-28v-46zm36 0v46h-28v-46zm-72 0v46h-28v-46zm36-54v46h-28v-46zm36 0v46h-28v-46zm-72 0v46h-28v-46z"/></svg>
`,Tn=m`
<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><g fill-rule="nonzero"><path d="m97.8975061 6h-65.7343743c-.6409315 0-1.1612995-.29021372-1.5611039-.87064117-.3998043-.58042745-.6004844-1.3048369-.60204-2.17322835 0-.81214848.20068-1.50731158.60204-2.08548931.4013601-.57817773.921728-.86839145 1.5611039-.87064117h65.7343743c.5600372 0 1.0508477.29021372 1.4724313.87064117s.6315976 1.27559055.6300505 2.08548931c0 .86839145-.2100226 1.5928009-.6300505 2.17322835-.420028.58042745-.9108384.87064117-1.4724313.87064117z"/><path d="m97.8975061 61h-65.7343743c-.6409315 0-1.1612995-.2902137-1.5611039-.8706412-.3998043-.5804274-.6004844-1.3048369-.60204-2.1732283 0-.8121485.20068-1.5073116.60204-2.0854893.4013601-.5781778.921728-.8683915 1.5611039-.8706412h65.7343743c.5600372 0 1.0508477.2902137 1.4724313.8706412.4215836.5804274.6315976 1.2755905.6300505 2.0854893 0 .8683914-.2100226 1.5928009-.6300505 2.1732283-.420028.5804275-.9108384.8706412-1.4724313.8706412z"/><path d="m97.8975061 19h-65.7343743c-.6409315 0-1.1612995-.2902137-1.5611039-.8706412-.3998043-.5804274-.6004844-1.3048369-.60204-2.1732283 0-.8121485.20068-1.5073116.60204-2.0854893.4013601-.5781778.921728-.8683915 1.5611039-.8706412h65.7343743c.5600372 0 1.0508477.2902137 1.4724313.8706412.4215836.5804274.6315976 1.2755905.6300505 2.0854893 0 .8683914-.2100226 1.5928009-.6300505 2.1732283-.420028.5804275-.9108384.8706412-1.4724313.8706412z"/><path d="m97.8975061 74h-65.7343743c-.6409315 0-1.1612995-.2902137-1.5611039-.8706412-.3998043-.5804274-.6004844-1.3048369-.60204-2.1732283 0-.8121485.20068-1.5073116.60204-2.0854893.4013601-.5781778.921728-.8683915 1.5611039-.8706412h65.7343743c.5600372 0 1.0508477.2902137 1.4724313.8706412.4215836.5804274.6315976 1.2755905.6300505 2.0854893 0 .8683914-.2100226 1.5928009-.6300505 2.1732283-.420028.5804275-.9108384.8706412-1.4724313.8706412z"/><path d="m97.8975061 32h-65.7343743c-.6409315 0-1.1612995-.2902137-1.5611039-.8706412-.3998043-.5804274-.6004844-1.3048369-.60204-2.1732283 0-.8121485.20068-1.5073116.60204-2.0854893.4013601-.5781778.921728-.8683915 1.5611039-.8706412h65.7343743c.5600372 0 1.0508477.2902137 1.4724313.8706412.4215836.5804274.6315976 1.2755905.6300505 2.0854893 0 .8683914-.2100226 1.5928009-.6300505 2.1732283-.420028.5804275-.9108384.8706412-1.4724313.8706412z"/><path d="m97.8975061 87h-65.7343743c-.6409315 0-1.1612995-.2902137-1.5611039-.8706412-.3998043-.5804274-.6004844-1.3048369-.60204-2.1732283 0-.8121485.20068-1.5073116.60204-2.0854893.4013601-.5781778.921728-.8683915 1.5611039-.8706412h65.7343743c.5600372 0 1.0508477.2902137 1.4724313.8706412.4215836.5804274.6315976 1.2755905.6300505 2.0854893 0 .8683914-.2100226 1.5928009-.6300505 2.1732283-.420028.5804275-.9108384.8706412-1.4724313.8706412z"/><path d="m97.8975061 45h-65.7343743c-.6409315 0-1.1612995-.2902137-1.5611039-.8706412-.3998043-.5804274-.6004844-1.3048369-.60204-2.1732283 0-.8121485.20068-1.5073116.60204-2.0854893.4013601-.5781778.921728-.8683915 1.5611039-.8706412h65.7343743c.5600372 0 1.0508477.2902137 1.4724313.8706412.4215836.5804274.6315976 1.2755905.6300505 2.0854893 0 .8683914-.2100226 1.5928009-.6300505 2.1732283-.420028.5804275-.9108384.8706412-1.4724313.8706412z"/><path d="m97.8975061 100h-65.7343743c-.6409315 0-1.1612995-.2902137-1.5611039-.8706412-.3998043-.5804274-.6004844-1.3048369-.60204-2.1732283 0-.8121485.20068-1.5073116.60204-2.0854893.4013601-.5781778.921728-.8683915 1.5611039-.8706412h65.7343743c.5600372 0 1.0508477.2902137 1.4724313.8706412.4215836.5804274.6315976 1.2755905.6300505 2.0854893 0 .8683914-.2100226 1.5928009-.6300505 2.1732283-.420028.5804275-.9108384.8706412-1.4724313.8706412z"/><path d="m0 0h25v25h-25z"/><path d="m0 55h25v25h-25z"/></g></svg>
`,En=m`
<svg viewBox="0 0 100 100" xmlns = "http://www.w3.org/2000/svg" > <path d="m96.9964435 6h-93.90621462c-.91561615 0-1.65899868-.29021372-2.23014758-.87064117-.57114891-.58042745-.85783455-1.3048369-.86005692-2.17322835 0-.81214848.28668564-1.50731158.86005692-2.08548931.57337127-.57817773 1.3167538-.86839145 2.23014758-.87064117h93.90621462c.800053 0 1.5012105.29021372 2.1034726.87064117.602262.58042745.9022819 1.27559055.9000718 2.08548931 0 .86839145-.3000321 1.5928009-.9000718 2.17322835-.6000398.58042745-1.3011973.87064117-2.1034726.87064117zm-93.90621462 9.6666667h93.90621462c.800053 0 1.5012105.2861891 2.1034726.8585673.602262.5723782.9022819 1.2579009.9000718 2.0565682 0 .8563487-.3000321 1.5851326-.9000718 2.1863516-.6000398.6012189-1.3011973.9007192-2.1034726.8985129h-93.90621462c-.91561615 0-1.65899868-.2995125-2.23014758-.8985129-.57114891-.5990005-.85783455-1.3277843-.86005692-2.1863516 0-.8008858.28668564-1.4864086.86005692-2.0565682.57337127-.5701597 1.3167538-.8563488 2.23014758-.8585673zm0 15.6700431h93.90621462c.800053 0 1.5012105.303883 2.1034726.9116489.602262.6077659.9022819 1.2886888.9000718 2.0427687-.0022346.7540799-.3022545 1.4496342-.9000718 2.0866629-.5978174.6370287-1.2989749.955543-2.1034726.955543h-93.90621462c-.85783454 0-1.58788286-.3038829-2.19014494-.9116488s-.90228193-1.3179516-.90007182-2.1305571c.00223463-.8126055.30225448-1.5081599.90007182-2.0866629.59781734-.5785031 1.32786566-.8688802 2.19014494-.8711312zm0 15.6632902h93.90621462c.800053 0 1.5012105.2861892 2.1034726.8585675.602262.5723783.9022819 1.2290603.9000718 1.9700462 0 .7986674-.3144775 1.5274514-.943408 2.186352-.6289306.6589006-1.3156427.9872417-2.0601364.9850343h-93.90621462c-.85783454 0-1.58788286-.3139318-2.19014494-.9417731-.60226208-.6278414-.90228193-1.3699365-.90007182-2.2262854 0-.7986674.2866979-1.4697699.86006918-2.0133074.57337127-.5435376 1.3167538-.8153063 2.23014758-.8153063zm0 15.6666667h93.90621462c.800053 0 1.5012105.3038947 2.1034726.9116593.602262.6077647.9022819 1.3472117.9000718 2.218341 0 .7540783-.3000321 1.4203685-.9000718 1.9988703-.6000398.5785019-1.3011973.8688784-2.1034726.8711294h-93.90621462c-.91561615 0-1.65899868-.2757451-2.23014758-.8272352-.57114891-.5514902-.85783455-1.2324117-.86005692-2.0427645 0-.8688784.28668564-1.6083253.86005692-2.218341.57337127-.6100156 1.3167538-.9138979 2.23014758-.9116593zm0 15.6666666h93.90621462c.800053 0 1.5012105.3038948 2.1034726.9116594.602262.6077646.9022819 1.3472116.9000718 2.2183409 0 .7540784-.3000321 1.4203685-.9000718 1.9988704-.6000398.5785019-1.3011973.8688784-2.1034726.8711293h-93.90621462c-.91561615 0-1.65899868-.275745-2.23014758-.8272352-.57114891-.5514901-.85783455-1.2324116-.86005692-2.0427645 0-.8688783.28668564-1.6083253.86005692-2.2183409.57337127-.6100156 1.3167538-.9138979 2.23014758-.9116594zm0 15.6666667h93.90621462c.800053 0 1.5012105.3038947 2.1034726.9116594.602262.6077646.9022819 1.3472116.9000718 2.2183409 0 .7540784-.3000321 1.4203685-.9000718 1.9988704-.6000398.5785019-1.3011973.8688783-2.1034726.8711293h-93.90621462c-.91561615 0-1.65899868-.2757451-2.23014758-.8272352-.57114891-.5514901-.85783455-1.2324116-.86005692-2.0427645 0-.8688783.28668564-1.6083253.86005692-2.2183409.57337127-.6100156 1.3167538-.913898 2.23014758-.9116594z" /> </svg>
`,Dn=class extends c{constructor(){super(...arguments),this.numResults=0}render(){let e=new Intl.NumberFormat(M()).format(this.numResults);return a`
      <div id="tooltip-container" role="tooltip">
        <div id="arrow"></div>
        <div id="tooltip-text">
          ${this.numResults===1?o(r`${e} result`):o(r`${e} results`)}
        </div>
      </div>
    `}static get styles(){let t=e`var(--tooltipArrowSize, 5px)`;return e`
      #tooltip-container {
        width: max-content;
        max-width: 200px;
        pointer-events: none;
      }

      #arrow {
        position: relative;
        left: calc(50% + ${e`var(--tooltipArrowOffset, 0px)`});
        top: 0;
        width: 0;
        height: 0;
        margin-top: calc(-1 * ${t});
        margin-left: calc(-1 * ${t});
        border: ${t} solid transparent;
        border-bottom-color: black;
      }

      #tooltip-text {
        padding: 3px 8px;
        border-radius: 4px;
        background-color: #000;
        color: white;
        font-size: 1.2rem;
        text-align: center;
        text-decoration: none;
      }
    `}};C([t({type:Number})],Dn.prototype,`numResults`,void 0),Dn=C([x(`alpha-bar-tooltip`),i()],Dn);var On,kn=(On=class extends c{constructor(){super(...arguments),this.selectedLetter=null,this.tooltipShown=!1,this.alphabet=`ABCDEFGHIJKLMNOPQRSTUVWXYZ`.split(``)}get selectedUppercaseLetter(){return this.selectedLetter?.toUpperCase()}render(){return a`
      <section id="container" aria-label=${this.ariaLandmarkLabel??n}>
        <ul>
          ${this.alphabet.map(e=>a`
              <li
                class=${e===this.selectedUppercaseLetter?`selected`:n}
                @mousemove=${this.handleMouseMove}
                @mouseleave=${this.handleMouseLeave}
              >
                ${this.letterButtonTemplate(e)}
                ${this.tooltipTemplate(e)}
              </li>
            `)}
        </ul>
      </section>
    `}letterButtonTemplate(e){let t=this.letterCounts?.[e]??0;return a`
      <button
        aria-label=${o(r`${e}: ${new Intl.NumberFormat(M()).format(t)} results`)}
        ?disabled=${!this.letterCounts?.[e]}
        @click=${()=>{this.letterClicked(e)}}
      >
        ${e}
      </button>
    `}tooltipTemplate(e){return this.hoveredLetter===e&&this.tooltipShown?a`<alpha-bar-tooltip
          data-letter=${e}
          .numResults=${this.letterCounts?.[this.hoveredLetter]??0}
        ></alpha-bar-tooltip>`:n}letterClicked(e){e===this.selectedUppercaseLetter?this.selectedLetter=null:this.selectedLetter=e,this.dispatchEvent(new CustomEvent(`letterChanged`,{detail:{selectedLetter:this.selectedUppercaseLetter}}))}async handleMouseMove(e){let t=e.target;if(t&&!this.tooltipShown){let e=t.textContent?.trim()??void 0;this.tooltipShown=!0,this.hoveredLetter=e,await this.updateComplete,await new Promise(e=>{setTimeout(e,0)}),this.tooltip&&this.tooltip.dataset.letter===e&&this.positionTooltip(t)}}handleMouseLeave(){this.tooltipShown=!1,this.hoveredLetter=void 0}positionTooltip(e){if(!this.tooltip)return;let t=this.tooltip.clientWidth,n=e.clientWidth/2-t/2,r=getComputedStyle(document.body),i=parseFloat(r.getPropertyValue(`margin-left`)),a=parseFloat(r.getPropertyValue(`margin-right`)),o=document.body.clientWidth+i+a,s=e.getBoundingClientRect().left+n,c=s+t,l;s<1?l=s-1:c>o-1&&(l=c-o+1),l&&(n-=l,this.tooltip.style.setProperty(`--tooltipArrowOffset`,`${l}px`)),this.tooltip.style.left=`${n}px`,this.tooltip.classList.add(`fade-in`)}},On.styles=e`
    h1 {
      font-size: 1.2rem;
    }

    #container {
      background-color: #ddd;
      color: #333;
      box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.5);
    }

    ul {
      list-style: none;
      display: flex;
      margin: 0;
      padding: 0.5rem 1rem;
      justify-content: space-between;
    }

    ul li {
      position: relative;
      flex: 1;
      text-align: center;
      max-width: 2.5rem;
      border-radius: 4px;
    }

    li:hover:not(.selected) button:not(:disabled) {
      background-color: #c0c0c0;
    }

    button {
      display: flex;
      justify-content: center;
      align-items: center;
      aspect-ratio: 1 / 1;
    }

    button {
      width: 100%;
      height: 100%;
      color: #333;
      appearance: none;
      background: none;
      border: none;
      border-radius: 4px;
      font-family: inherit;
      font-size: 1.2rem;
      cursor: pointer;
    }

    button:disabled {
      color: #aaa;
      cursor: default;
    }

    .selected {
      background-color: #2c2c2c;
    }

    .selected button {
      color: white;
    }

    alpha-bar-tooltip {
      position: absolute;
      top: 100%;
      left: -9999px;
      margin-top: 3px;
      z-index: 1;

      opacity: 0;
      transition: opacity 0.2s ease;
    }

    alpha-bar-tooltip.fade-in {
      opacity: 1;
    }

    /* Make alphabet bar a 2-row container in small screen widths */
    @media screen and (max-width: 768px) {
      ul {
        display: grid;
        grid-template-columns: repeat(13, 1fr);
      }
    }
  `,On);C([t({type:String})],kn.prototype,`selectedLetter`,void 0),C([t({type:Object})],kn.prototype,`letterCounts`,void 0),C([t({type:String})],kn.prototype,`ariaLandmarkLabel`,void 0),C([s()],kn.prototype,`tooltipShown`,void 0),C([s()],kn.prototype,`hoveredLetter`,void 0),C([S(`alpha-bar-tooltip`)],kn.prototype,`tooltip`,void 0),kn=C([x(`alpha-bar`),i()],kn);var V=class extends c{constructor(){super(...arguments),this.defaultSortDirection=null,this.defaultSortField=E.relevance,this.sortDirection=null,this.selectedSort=E.default,this.selectedTitleFilter=null,this.selectedCreatorFilter=null,this.sortFieldAvailability=ve,this.enableSortOptionsSlot=!1,this.suppressDisplayModes=!1,this.alphaSelectorVisible=null,this.dropdownBackdropVisible=!1,this.boundSortBarSelectorEscapeListener=e=>{e.key===`Escape`&&this.closeDropdown()}}render(){return a`
      <div id="container">
        <div id="sort-bar-row">
          <section id="sort-bar" aria-label=${o(`Sorting options`)}>
            <slot name="sort-options-left"></slot>
            <div id="sort-options">
              ${this.enableSortOptionsSlot?a`<slot name="sort-options"></slot>`:a`
                    <div class="sort-direction-container">
                      ${this.sortDirectionSelectorTemplate}
                    </div>
                    <span class="sort-by-text">${o(`Sort by:`)}</span>
                    <div id="sort-selector-container">
                      ${this.sortSelectorTemplate}
                    </div>
                  `}
            </div>
            <slot name="sort-options-right"></slot>

            ${this.suppressDisplayModes?n:a`<div id="display-style-selector">
                  ${this.displayOptionTemplate}
                </div>`}
          </section>
          <slot name="sort-bar-end"></slot>
        </div>

        ${this.dropdownBackdropVisible?this.dropdownBackdrop:n}
        ${this.alphaBarTemplate}
      </div>
    `}willUpdate(e){if((e.has(`selectedSort`)||e.has(`defaultSortField`))&&this.selectedSort&&this.selectedSort!==E.default&&this.sortDirection===null){let e=D[this.finalizedSortField];this.sortDirection=e.defaultSortDirection}}updated(e){e.has(`displayMode`)&&this.displayModeChanged(),e.has(`selectedTitleFilter`)&&this.selectedTitleFilter&&(this.alphaSelectorVisible=`title`),e.has(`selectedCreatorFilter`)&&this.selectedCreatorFilter&&(this.alphaSelectorVisible=`creator`),e.has(`dropdownBackdropVisible`)&&this.setupEscapeListeners()}setupEscapeListeners(){this.dropdownBackdropVisible?document.addEventListener(`keydown`,this.boundSortBarSelectorEscapeListener):document.removeEventListener(`keydown`,this.boundSortBarSelectorEscapeListener)}get alphaBarTemplate(){if(![`title`,`creator`].includes(this.selectedSort))return n;if(this.alphaSelectorVisible===null){if(this.selectedSort===`creator`)return this.creatorSelectorBar;if(this.selectedSort===`title`)return this.titleSelectorBar}else return this.alphaSelectorVisible===`creator`?this.creatorSelectorBar:this.titleSelectorBar;return n}get sortDirectionSelectorTemplate(){let e=this.sortDirection===`asc`?o(`descending`):o(`ascending`),t=this.canChangeSortDirection?o(r`Change to ${e} sort`):o(`Directions are not available for the current sort option`);return a`
      <button
        class="sort-direction-selector"
        title=${t}
        ?disabled=${!this.canChangeSortDirection}
        @click=${this.handleSortDirectionClicked}
      >
        <span class="sr-only">${t}</span>
        ${this.sortDirectionIcon}
      </button>
    `}get sortDirectionIcon(){return this.canChangeSortDirection?a`
      <div class="sort-direction-icon">
        ${this.finalizedSortDirection===`asc`?xn:Sn}
      </div>
    `:a`<div class="sort-direction-icon">${Cn}</div>`}get sortSelectorTemplate(){let e=Object.values(D).filter(e=>e.shownInSortBar&&this.sortFieldAvailability[e.field]);return a`
      <div id="sort-dropdown-container">
        ${this.getSortDropdown({displayName:D[this.finalizedSortField].displayName,id:`sort-dropdown`,selected:!0,dropdownOptions:e.map(e=>this.getDropdownOption(e.field)),selectedOption:this.finalizedSortField,onOptionSelected:this.sortOptionSelected,onDropdownClick:()=>{this.dropdownBackdropVisible=this.sortOptionsDropdown.open,this.sortOptionsDropdown.classList.toggle(`open`,this.sortOptionsDropdown.open)}})}
      </div>
    `}getSortDropdown(e){return a`
      <ia-dropdown
        id=${e.id}
        class=${e.selected?`selected`:``}
        displayCaret
        closeOnSelect
        includeSelectedOption
        .openViaButton=${e.selected}
        .options=${e.dropdownOptions}
        .selectedOption=${e.selectedOption??``}
        @optionSelected=${e.onOptionSelected??n}
        @click=${e.onDropdownClick??n}
      >
        <span
          class="dropdown-label"
          slot="dropdown-label"
          data-title=${e.displayName}
        >
          ${e.displayName}
        </span>
      </ia-dropdown>
    `}getDropdownOption(e){return{id:e,label:a`
        <span class="dropdown-option-label">
          ${D[e].displayName}
        </span>
      `}}sortOptionSelected(e){this.dropdownBackdropVisible=!1;let t=e.detail.option.id;this.setSelectedSort(t),this.alphaSelectorVisible=null,t!==`title`&&this.selectedTitleFilter&&(this.selectedTitleFilter=null,this.emitTitleLetterChangedEvent()),t!==`creator`&&this.selectedCreatorFilter&&(this.selectedCreatorFilter=null,this.emitCreatorLetterChangedEvent())}get displayOptionTemplate(){return a`
      <ul>
        <li>
          <button
            id="grid-button"
            @click=${()=>{this.displayMode=`grid`}}
            class=${this.displayMode===`grid`?`active`:``}
            title=${o(`Tile view`)}
            data-testid="grid-button"
          >
            ${wn}
          </button>
        </li>
        <li>
          <button
            id="list-detail-button"
            @click=${()=>{this.displayMode=`list-detail`}}
            class=${this.displayMode===`list-detail`?`active`:``}
            title=${o(`List view`)}
            data-testid="list-detail-button"
          >
            ${Tn}
          </button>
        </li>
        <li>
          <button
            id="list-compact-button"
            @click=${()=>{this.displayMode=`list-compact`}}
            class=${this.displayMode===`list-compact`?`active`:``}
            title=${o(`Compact list view`)}
            data-testid="list-compact-button"
          >
            ${En}
          </button>
        </li>
      </ul>
    `}get dropdownBackdrop(){return a`
      <div
        id="sort-selector-backdrop"
        @keyup=${this.closeDropdown}
        @click=${this.closeDropdown}
      ></div>
    `}closeDropdown(){this.dropdownBackdropVisible=!1,this.sortOptionsDropdown&&(this.sortOptionsDropdown.open=!1,this.sortOptionsDropdown.classList.remove(`open`))}setSortDirection(e){this.sortDirection=e,this.emitSortChangedEvent()}toggleSortDirection(){this.setSortDirection(this.finalizedSortDirection===`desc`?`asc`:`desc`)}handleSortDirectionClicked(){!this.sortDirection&&this.defaultSortField&&this.defaultSortDirection&&(this.selectedSort=this.defaultSortField,this.sortDirection=this.defaultSortDirection),this.toggleSortDirection()}setSelectedSort(e){this.selectedSort=e;let t=D[e];this.sortDirection=t.defaultSortDirection,this.emitSortChangedEvent()}get finalizedSortField(){let e=this.selectedSort===E.default?this.defaultSortField:this.selectedSort;return this.sortFieldAvailability[e]?e:this.firstAvailableOption?.field??e}get finalizedSortDirection(){return this.sortDirection===null?this.defaultSortDirection:this.sortDirection}get firstAvailableOption(){return Object.values(D).find(e=>e.shownInSortBar&&this.sortFieldAvailability[e.field])}get canChangeSortDirection(){return D[this.finalizedSortField].canSetDirection}get titleSelectorBar(){return a` <alpha-bar
      .selectedLetter=${this.selectedTitleFilter}
      .letterCounts=${this.prefixFilterCountMap?.title}
      ariaLandmarkLabel=${o(`Filter by title letter`)}
      @letterChanged=${this.titleLetterChanged}
    ></alpha-bar>`}get creatorSelectorBar(){return a` <alpha-bar
      .selectedLetter=${this.selectedCreatorFilter}
      .letterCounts=${this.prefixFilterCountMap?.creator}
      ariaLandmarkLabel=${o(`Filter by creator letter`)}
      @letterChanged=${this.creatorLetterChanged}
    ></alpha-bar>`}titleLetterChanged(e){this.selectedTitleFilter=e.detail.selectedLetter??null,this.emitTitleLetterChangedEvent()}creatorLetterChanged(e){this.selectedCreatorFilter=e.detail.selectedLetter??null,this.emitCreatorLetterChangedEvent()}emitTitleLetterChangedEvent(){let e=new CustomEvent(`titleLetterChanged`,{detail:{selectedLetter:this.selectedTitleFilter}});this.dispatchEvent(e)}emitCreatorLetterChangedEvent(){let e=new CustomEvent(`creatorLetterChanged`,{detail:{selectedLetter:this.selectedCreatorFilter}});this.dispatchEvent(e)}displayModeChanged(){let e=new CustomEvent(`displayModeChanged`,{detail:{displayMode:this.displayMode}});this.dispatchEvent(e)}emitSortChangedEvent(){let e=new CustomEvent(`sortChanged`,{detail:{selectedSort:this.selectedSort,sortDirection:this.sortDirection}});this.dispatchEvent(e)}static get styles(){let t=e`#bbbbbb`;return[j,e`
        #container {
          position: relative;
        }

        /*
         * The sort-bar-end slot shares this row with the sort bar but sits
         * outside its "Sorting options" landmark. When the slotted content
         * doesn't fit beside the sort controls, it wraps onto its own line at
         * the right rather than overflowing the row.
         */
        #sort-bar-row {
          display: flex;
          flex-wrap: wrap;
          justify-content: flex-end;
          align-items: center;
          padding-bottom: 1px;
          border-bottom: 1px solid #2c2c2c;
          font-size: 1.4rem;
        }

        #sort-bar {
          flex: auto;
          min-width: 0;
          display: flex;
          justify-content: flex-start;
          align-items: center;
        }

        #sort-options {
          display: flex;
          align-items: center;
          flex-grow: 1;
        }

        ul {
          list-style: none;
          display: flex;
          align-items: center;
          margin: 0;
          padding: 0;
        }

        li {
          padding: 0;
        }

        .sort-by-text {
          margin-right: 5px;
          font-weight: bold;
          white-space: nowrap;
        }

        .sort-direction-container {
          display: flex;
          align-self: stretch;
          flex: 0;
          margin: 0 3px;
        }

        .sort-direction-selector {
          display: flex;
          justify-content: center;
          width: 30px;
          margin: 0 5px 0 0;
          padding: 7px 8px;
          max-height: fit-content;
          border-radius: 5px;
          background: white;
          border: 1px solid rgb(25, 72, 128);
          appearance: none;
          cursor: pointer;
        }

        .sort-direction-selector:disabled {
          cursor: not-allowed;
          border-color: ${t};
        }

        .sort-direction-icon {
          display: flex;
          align-items: center;
          background: none;
          color: inherit;
          border: none;
          padding: 0;
          outline: inherit;
          width: 12px;
          height: 12px;
        }

        .sort-direction-icon > svg {
          flex: 1;
        }

        #sort-selector-container {
          flex: 1;
          display: flex;
          justify-content: flex-start;
          align-items: center;
        }

        #sort-dropdown-container {
          display: flex;
          justify-content: flex-start;
          align-items: center;
        }

        #sort-selector-backdrop {
          position: fixed;
          top: 0;
          left: 0;
          width: 100vw;
          height: 100vh;
          z-index: 1;
          background-color: transparent;
        }

        #display-style-selector {
          flex: 0;
        }

        #display-style-selector button {
          background: none;
          color: inherit;
          border: none;
          appearance: none;
          cursor: pointer;
          -webkit-appearance: none;
          fill: ${t};
        }

        #display-style-selector button.active {
          fill: var(--ia-theme-primary-text-color, #2c2c2c);
        }

        #display-style-selector button svg {
          width: 24px;
          height: 24px;
        }

        ia-dropdown {
          --dropdownTextColor: white;
          --dropdownOffsetTop: 0;
          --dropdownBorderTopWidth: 0;
          --dropdownBorderTopLeftRadius: 0;
          --dropdownBorderTopRightRadius: 0;
          --dropdownWhiteSpace: nowrap;
          --dropdownListZIndex: 2;
          --dropdownCaretColor: var(--ia-theme-primary-text-color, #2c2c2c);
          --dropdownSelectedTextColor: white;
          --dropdownSelectedBgColor: rgba(255, 255, 255, 0.3);
          --dropdownHoverBgColor: rgba(255, 255, 255, 0.3);
          --caretHeight: 9px;
          --caretWidth: 12px;
          --caretPadding: 0 5px 0 0;
        }
        ia-dropdown.selected .dropdown-label {
          font-weight: bold;
        }
        ia-dropdown.open {
          z-index: 2;
        }

        .dropdown-label {
          display: inline-block;
          height: 100%;
          padding-left: 5px;
          font-size: 1.4rem;
          font-family: var(--ia-theme-base-font-family);
          line-height: 2;
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          white-space: nowrap;
          user-select: none;
        }
      `]}};C([t({type:String})],V.prototype,`displayMode`,void 0),C([t({type:String})],V.prototype,`defaultSortDirection`,void 0),C([t({type:String})],V.prototype,`defaultSortField`,void 0),C([t({type:String})],V.prototype,`sortDirection`,void 0),C([t({type:String})],V.prototype,`selectedSort`,void 0),C([t({type:String})],V.prototype,`selectedTitleFilter`,void 0),C([t({type:String})],V.prototype,`selectedCreatorFilter`,void 0),C([t({type:Object})],V.prototype,`sortFieldAvailability`,void 0),C([t({type:Boolean,reflect:!0})],V.prototype,`enableSortOptionsSlot`,void 0),C([t({type:Boolean,reflect:!0})],V.prototype,`suppressDisplayModes`,void 0),C([t({type:Object})],V.prototype,`prefixFilterCountMap`,void 0),C([s()],V.prototype,`alphaSelectorVisible`,void 0),C([s()],V.prototype,`dropdownBackdropVisible`,void 0),C([S(`#sort-dropdown`)],V.prototype,`sortOptionsDropdown`,void 0),V=C([x(`sort-filter-bar`),i()],V);var An=e`var(--white, #fff)`,jn=e`var(--primaryDisableCTAFill, #767676)`,Mn=e`var(--secondaryCTABorder, #999)`,Nn=e`var(--primaryCTAFill, #194880)`,Pn=e`var(--primaryCTAFillRGB, 25, 72, 128)`,Fn=e`var(--primaryCTABorder, #c5d1df)`,In=e`var(--primaryErrorCTAFill, #d9534f)`,Ln=e`var(--primaryErrorCTAFillRGB, 229, 28, 38)`,Rn=e`var(--primaryErrorCTABorder, #d43f3a)`,zn=e`var(--secondaryCTAFill, #333)`,Bn=e`var(--secondaryCTAFillRGB, 51, 51, 51)`,Vn=e`var(--primaryCTABorder, #979797)`,Hn=e`var(---primaryWarningFill, #ee8950)`,Un=e`var(--primaryWarningFillRGB, 238, 137, 80)`,Wn=e`
  .ia-button {
    min-height: 3rem;
    cursor: pointer;
    color: ${An};
    line-height: normal;
    border-radius: 0.4rem;
    font-size: 1.4rem;
    font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
    border: 1px solid transparent;
    white-space: nowrap;
    appearance: auto;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    transition: all 0.1s ease 0s;
    vertical-align: middle;
    padding: 0 3rem;
    outline-color: ${An};
    outline-offset: -4px;
    user-select: none;
    text-decoration: none;
    width: fit-content;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    -o-user-select: none;
  }
  .ia-button:focus-visible {
    outline-style: double;
  }
  .ia-button:disabled {
    cursor: not-allowed;
    background-color: ${jn};
    border: 1px solid ${Mn};
  }
  .ia-button.transparent {
    background-color: transparent;
  }

  .ia-button.primary:disabled,
  .ia-button.danger:disabled,
  .ia-button.warning:disabled,
  .ia-button.dark:disabled {
    opacity: 0.5;
  }

  .ia-button.primary {
    background-color: ${Nn};
    border-color: ${Fn};
  }
  .ia-button.primary:hover {
    background-color: rgba(${Pn}, 0.9);
  }
  .ia-button.primary:focus-visible {
    background-color: rgba(${Pn}, 0.8);
  }
  .ia-button.primary:active {
    background-color: rgba(${Pn}, 0.7);
  }

  .ia-button.danger {
    background-color: ${In};
    border-color: ${Rn};
  }
  .ia-button.danger:hover {
    background-color: rgba(${Ln}, 0.9);
  }
  .ia-button.danger:focus-visible {
    background-color: rgba(${Ln}, 0.8);
  }
  .ia-button.danger:active {
    background-color: rgba(${Ln}, 0.7);
  }

  .ia-button.warning {
    background-color: ${Hn};
    border-color: ${e`var(--primaryWarningBorder, #ec7939)`};
  }
  .ia-button.warning:hover {
    background-color: rgba(${Un}, 0.9);
  }
  .ia-button.warning:focus-visible {
    background-color: rgba(${Un}, 0.8);
  }
  .ia-button.warning:active {
    background-color: rgba(${Un}, 0.7);
  }

  .ia-button.dark {
    background-color: ${zn};
    border-color: ${Vn};
  }
  .ia-button.dark:hover {
    background-color: rgba(${Bn}, 0.9);
  }
  .ia-button.dark:focus-visible {
    background-color: rgba(${Bn}, 0.8);
  }
  .ia-button.dark:active {
    background-color: rgba(${Bn}, 0.7);
  }

  .ia-button.link {
    margin: 0;
    padding: 6px;
    border: 0;
    appearance: none;
    background: none;
    color: var(--ia-theme-link-color, #4b64ff);
    text-decoration: none;
    cursor: pointer;
  }
  .ia-button.link:hover {
    text-decoration: underline;
  }
`,Gn=class extends c{constructor(){super(...arguments),this.items=[]}render(){return a`
      <ul>
        ${y(this.items,({title:e,date:t})=>a`
            <li>
              <span class="item-title">${e??`[untitled]`}</span>
              <span class="item-date">${t??``}</span>
            </li>
          `)}
      </ul>
      ${this.message?a`<p class="message">${this.message}</p>`:n}
      <div class="button-bar">
        <button class="remove-items-btn" @click=${this.removeItemsBtnClicked}>
          ${o(`Remove items`)}
        </button>
      </div>
    `}removeItemsBtnClicked(){this.dispatchEvent(new CustomEvent(`confirm`,{detail:{items:this.items}}))}static get styles(){return e`
      ul {
        margin: 0;
        padding: 0 10px;
        font-size: 1.4rem;
        list-style-type: none;
        max-height: min(400px, 40vh);
        overflow-y: auto;
      }

      li {
        display: flex;
        justify-content: space-between;
        padding: 2px 0;
      }
      li:not(:last-of-type) {
        border-bottom: 1px solid rgb(232, 232, 232);
      }

      .item-title {
        word-break: break-word;
      }

      .item-date {
        white-space: nowrap;
      }

      .message {
        font-size: 1.4rem;
        padding: 5px 10px;
      }

      .button-bar {
        display: flex;
        justify-content: center;
        margin: 10px 5px;
      }

      .remove-items-btn {
        margin-bottom: 10px;
        padding: 10px;
        border: 1px solid var(--primaryErrorCTABorder, #d43f3a);
        border-radius: 4px;
        color: white;
        background: var(--primaryErrorCTAFill, #d9534f);
        appearance: none;
        cursor: pointer;
      }
      .remove-items-btn:hover {
        background: rgba(var(--primaryErrorCTAFillRGB, 229, 28, 38), 0.9);
      }
      .remove-items-btn:active {
        background: rgba(var(--primaryErrorCTAFillRGB, 229, 28, 38), 0.7);
      }
    `}};C([t({type:Object})],Gn.prototype,`items`,void 0),C([t({type:String})],Gn.prototype,`message`,void 0),Gn=C([x(`remove-items-modal-content`),i()],Gn);var H=class extends c{constructor(){super(...arguments),this.selectedItems=[],this.showSelectAll=!1,this.showUnselectAll=!1,this.showItemManageButton=!1,this.removeAllowed=!1}render(){return a`
      <div class="manage-container">
        <span class="manage-label"
          >${this.label??o(`Select items to remove`)}</span
        >
        <div class="manage-buttons">
          <button class="ia-button dark" @click=${this.cancelClicked}>
            ${o(`Cancel`)}
          </button>
          <button
            class="ia-button danger"
            ?disabled=${!this.removeAllowed}
            @click=${this.showRemoveItemsModal}
          >
            ${o(`Remove selected items`)} (${this.selectedItems.length})...
          </button>
          ${l(this.showItemManageButton,()=>a` <button
                class="ia-button warning"
                ?disabled=${!this.removeAllowed}
                @click=${this.manageItemsClicked}
              >
                ${o(`Item Manager the items`)} (${this.selectedItems.length})
              </button>`)}
          <div class="selection-buttons">
            ${l(this.showSelectAll,()=>a` <button
                  class="ia-button link select-all-btn"
                  @click=${this.selectAllClicked}
                >
                  ${o(`Select all`)}
                </button>`)}
            ${l(this.showUnselectAll,()=>a` <button
                  class="ia-button link unselect-all-btn"
                  @click=${this.unselectAllClicked}
                >
                  ${o(`Unselect all`)}
                </button>`)}
          </div>
        </div>
      </div>
    `}get manageViewModalMsg(){let e=this.selectedItems.length>1;switch(this.profileElement){case`uploads`:return o(e?`Note: It may take a few minutes for these items to stop appearing in your uploads list.`:`Note: It may take a few minutes for this item to stop appearing in your uploads list.`);case`web_archives`:return o(e?`Note: It may take a few minutes for these items to stop appearing in your web archives list.`:`Note: It may take a few minutes for this item to stop appearing in your web archives list.`);case`favorites`:return o(e?`Note: It may take a few minutes for these items to stop appearing in your favorites list.`:`Note: It may take a few minutes for this item to stop appearing in your favorites list.`);default:return``}}cancelClicked(){this.dispatchEvent(new CustomEvent(`cancel`))}removeItemsClicked(){this.dispatchEvent(new CustomEvent(`removeItems`))}manageItemsClicked(){this.dispatchEvent(new CustomEvent(`manageItems`))}selectAllClicked(){this.dispatchEvent(new CustomEvent(`selectAll`))}unselectAllClicked(){this.dispatchEvent(new CustomEvent(`unselectAll`))}showRemoveItemsModal(){let e=a`
      <remove-items-modal-content
        .items=${this.selectedItems}
        .message=${this.manageViewModalMsg}
        @confirm=${()=>this.removeItemsClicked()}
      ></remove-items-modal-content>
    `,t=new oe({showProcessingIndicator:!1,processingImageMode:`processing`,bodyColor:`#fff`,headerColor:`#194880`,showHeaderLogo:!1,closeOnBackdropClick:!0,title:a`${o(`Are you sure you want to remove these items?`)}`});this.modalManager?.classList.add(`remove-items`),this.modalManager?.showModal({config:t,customModalContent:e,userClosedModalCallback:()=>{this.modalManager?.classList.remove(`remove-items`)}})}showRemoveItemsProcessingModal(){let e=new oe({showProcessingIndicator:!0,processingImageMode:`processing`,bodyColor:`#fff`,headerColor:`#194880`,showHeaderLogo:!1,closeOnBackdropClick:!0,title:a`${o(`Removing selected items...`)}`});this.modalManager?.classList.add(`remove-items`),this.modalManager?.showModal({config:e,userClosedModalCallback:()=>{this.modalManager?.classList.remove(`remove-items`)}})}showRemoveItemsErrorModal(){let e=new oe({showProcessingIndicator:!1,processingImageMode:`processing`,bodyColor:`#fff`,headerColor:`#691916`,showHeaderLogo:!1,closeOnBackdropClick:!0,title:a`${o(`Error: unable to remove items`)}`,message:a`${o(`An error occurred while removing items. Please try again in a few minutes.`)}`});this.modalManager?.classList.add(`remove-items`),this.modalManager?.showModal({config:e,userClosedModalCallback:()=>{this.modalManager?.classList.remove(`remove-items`)}})}static get styles(){return e`
      ${Wn}
      .manage-container {
        display: flex;
        align-items: center;
        column-gap: 5px;
        padding: 20px 0 20px;
        flex-wrap: wrap;
      }

      .manage-label {
        display: inline-block;
        font-weight: bold;
        font-size: 1.8rem;
        padding-right: 10px;
      }

      .manage-buttons {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        column-gap: 5px;
      }

      .selection-buttons {
        display: inherit;
      }

      .ia-button,
      button {
        padding: 6px 12px;
        font-size: 1.4rem;
        margin: 3px 0;
      }
    `}};C([t({type:String})],H.prototype,`label`,void 0),C([t({type:Object})],H.prototype,`modalManager`,void 0),C([t({type:Array})],H.prototype,`selectedItems`,void 0),C([t({type:String})],H.prototype,`profileElement`,void 0),C([t({type:Boolean})],H.prototype,`showSelectAll`,void 0),C([t({type:Boolean})],H.prototype,`showUnselectAll`,void 0),C([t({type:Boolean})],H.prototype,`showItemManageButton`,void 0),C([t({type:Boolean})],H.prototype,`removeAllowed`,void 0),H=C([x(`manage-bar`),i()],H);var Kn=3600,qn=Kn*24,Jn=qn*7,Yn=1e3,Xn=60*Yn,Zn=Kn*Yn,Qn=qn*Yn,$n=Jn*Yn,er=`millisecond`,tr=`second`,nr=`minute`,rr=`hour`,ir=`week`,ar=`month`,or=`quarter`,sr=`year`,cr=`date`,lr=`Invalid Date`,ur=/^(\d{4})[-/]?(\d{1,2})?[-/]?(\d{0,2})[Tt\s]*(\d{1,2})?:?(\d{1,2})?:?(\d{1,2})?[.:]?(\d+)?$/,dr=/\[([^\]]+)]|YYYY|YY|M{1,4}|D{1,2}|d{1,4}|H{1,2}|h{1,2}|a|A|m{1,2}|s{1,2}|Z{1,2}|SSS/g,fr={name:`en`,weekdays:`Sunday_Monday_Tuesday_Wednesday_Thursday_Friday_Saturday`.split(`_`),months:`January_February_March_April_May_June_July_August_September_October_November_December`.split(`_`),ordinal:function(e){var t=[`th`,`st`,`nd`,`rd`],n=e%100;return`[`+e+(t[(n-20)%10]||t[n]||t[0])+`]`}},pr=function(e,t,n){var r=String(e);return!r||r.length>=t?e:``+Array(t+1-r.length).join(n)+e},mr={s:pr,z:function(e){var t=-e.utcOffset(),n=Math.abs(t),r=Math.floor(n/60),i=n%60;return(t<=0?`+`:`-`)+pr(r,2,`0`)+`:`+pr(i,2,`0`)},m:function e(t,n){if(t.date()<n.date())return-e(n,t);var r=(n.year()-t.year())*12+(n.month()-t.month()),i=t.clone().add(r,ar),a=n-i<0,o=t.clone().add(r+(a?-1:1),ar);return+(-(r+(n-i)/(a?i-o:o-i))||0)},a:function(e){return e<0?Math.ceil(e)||0:Math.floor(e)},p:function(e){return{M:`month`,y:`year`,w:`week`,d:`day`,D:`date`,h:`hour`,m:`minute`,s:`second`,ms:`millisecond`,Q:`quarter`}[e]||String(e||``).toLowerCase().replace(/s$/,``)},u:function(e){return e===void 0}},hr=`en`,gr={};gr[hr]=fr;var _r=`$isDayjsObject`,vr=function(e){return e instanceof Sr||!!(e&&e[_r])},yr=function e(t,n,r){var i;if(!t)return hr;if(typeof t==`string`){var a=t.toLowerCase();gr[a]&&(i=a),n&&(gr[a]=n,i=a);var o=t.split(`-`);if(!i&&o.length>1)return e(o[0])}else{var s=t.name;gr[s]=t,i=s}return!r&&i&&(hr=i),i||!r&&hr},U=function(e,t){if(vr(e))return e.clone();var n=typeof t==`object`?t:{};return n.date=e,n.args=arguments,new Sr(n)},br=function(e,t){return U(e,{locale:t.$L,utc:t.$u,x:t.$x,$offset:t.$offset})},W=mr;W.l=yr,W.i=vr,W.w=br;var xr=function(e){var t=e.date,n=e.utc;if(t===null)return new Date(NaN);if(W.u(t))return new Date;if(t instanceof Date)return new Date(t);if(typeof t==`string`&&!/Z$/i.test(t)){var r=t.match(ur);if(r){var i=r[2]-1||0,a=(r[7]||`0`).substring(0,3);return n?new Date(Date.UTC(r[1],i,r[3]||1,r[4]||0,r[5]||0,r[6]||0,a)):new Date(r[1],i,r[3]||1,r[4]||0,r[5]||0,r[6]||0,a)}}return new Date(t)},Sr=function(){function e(e){this.$L=yr(e.locale,null,!0),this.parse(e),this.$x=this.$x||e.x||{},this[_r]=!0}var t=e.prototype;return t.parse=function(e){this.$d=xr(e),this.init()},t.init=function(){var e=this.$d;this.$y=e.getFullYear(),this.$M=e.getMonth(),this.$D=e.getDate(),this.$W=e.getDay(),this.$H=e.getHours(),this.$m=e.getMinutes(),this.$s=e.getSeconds(),this.$ms=e.getMilliseconds()},t.$utils=function(){return W},t.isValid=function(){return this.$d.toString()!==lr},t.isSame=function(e,t){var n=U(e);return this.startOf(t)<=n&&n<=this.endOf(t)},t.isAfter=function(e,t){return U(e)<this.startOf(t)},t.isBefore=function(e,t){return this.endOf(t)<U(e)},t.$g=function(e,t,n){return W.u(e)?this[t]:this.set(n,e)},t.unix=function(){return Math.floor(this.valueOf()/1e3)},t.valueOf=function(){return this.$d.getTime()},t.startOf=function(e,t){var n=this,r=W.u(t)?!0:t,i=W.p(e),a=function(e,t){var i=W.w(n.$u?Date.UTC(n.$y,t,e):new Date(n.$y,t,e),n);return r?i:i.endOf(`day`)},o=function(e,t){return W.w(n.toDate()[e].apply(n.toDate(`s`),(r?[0,0,0,0]:[23,59,59,999]).slice(t)),n)},s=this.$W,c=this.$M,l=this.$D,u=`set`+(this.$u?`UTC`:``);switch(i){case sr:return r?a(1,0):a(31,11);case ar:return r?a(1,c):a(0,c+1);case ir:var d=this.$locale().weekStart||0,f=(s<d?s+7:s)-d;return a(r?l-f:l+(6-f),c);case`day`:case cr:return o(u+`Hours`,0);case rr:return o(u+`Minutes`,1);case nr:return o(u+`Seconds`,2);case tr:return o(u+`Milliseconds`,3);default:return this.clone()}},t.endOf=function(e){return this.startOf(e,!1)},t.$set=function(e,t){var n,r=W.p(e),i=`set`+(this.$u?`UTC`:``),a=(n={},n.day=i+`Date`,n[cr]=i+`Date`,n[ar]=i+`Month`,n[sr]=i+`FullYear`,n[rr]=i+`Hours`,n[nr]=i+`Minutes`,n[tr]=i+`Seconds`,n[er]=i+`Milliseconds`,n)[r],o=r===`day`?this.$D+(t-this.$W):t;if(r===`month`||r===`year`){var s=this.clone().set(cr,1);s.$d[a](o),s.init(),this.$d=s.set(cr,Math.min(this.$D,s.daysInMonth())).$d}else a&&this.$d[a](o);return this.init(),this},t.set=function(e,t){return this.clone().$set(e,t)},t.get=function(e){return this[W.p(e)]()},t.add=function(e,t){var n=this,r;e=Number(e);var i=W.p(t),a=function(t){var r=U(n);return W.w(r.date(r.date()+Math.round(t*e)),n)};if(i===`month`)return this.set(ar,this.$M+e);if(i===`year`)return this.set(sr,this.$y+e);if(i===`day`)return a(1);if(i===`week`)return a(7);var o=(r={},r.minute=6e4,r.hour=36e5,r.second=1e3,r)[i]||1,s=this.$d.getTime()+e*o;return W.w(s,this)},t.subtract=function(e,t){return this.add(e*-1,t)},t.format=function(e){var t=this,n=this.$locale();if(!this.isValid())return n.invalidDate||`Invalid Date`;var r=e||`YYYY-MM-DDTHH:mm:ssZ`,i=W.z(this),a=this.$H,o=this.$m,s=this.$M,c=n.weekdays,l=n.months,u=n.meridiem,d=function(e,n,i,a){return e&&(e[n]||e(t,r))||i[n].slice(0,a)},f=function(e){return W.s(a%12||12,e,`0`)},p=u||function(e,t,n){var r=e<12?`AM`:`PM`;return n?r.toLowerCase():r},m=function(e){switch(e){case`YY`:return String(t.$y).slice(-2);case`YYYY`:return W.s(t.$y,4,`0`);case`M`:return s+1;case`MM`:return W.s(s+1,2,`0`);case`MMM`:return d(n.monthsShort,s,l,3);case`MMMM`:return d(l,s);case`D`:return t.$D;case`DD`:return W.s(t.$D,2,`0`);case`d`:return String(t.$W);case`dd`:return d(n.weekdaysMin,t.$W,c,2);case`ddd`:return d(n.weekdaysShort,t.$W,c,3);case`dddd`:return c[t.$W];case`H`:return String(a);case`HH`:return W.s(a,2,`0`);case`h`:return f(1);case`hh`:return f(2);case`a`:return p(a,o,!0);case`A`:return p(a,o,!1);case`m`:return String(o);case`mm`:return W.s(o,2,`0`);case`s`:return String(t.$s);case`ss`:return W.s(t.$s,2,`0`);case`SSS`:return W.s(t.$ms,3,`0`);case`Z`:return i;default:break}return null};return r.replace(dr,function(e,t){return t||m(e)||i.replace(`:`,``)})},t.utcOffset=function(){return-Math.round(this.$d.getTimezoneOffset()/15)*15},t.diff=function(e,t,n){var r=this,i=W.p(t),a=U(e),o=(a.utcOffset()-this.utcOffset())*Xn,s=this-a,c=function(){return W.m(r,a)},l;switch(i){case sr:l=c()/12;break;case ar:l=c();break;case or:l=c()/3;break;case ir:l=(s-o)/$n;break;case`day`:l=(s-o)/Qn;break;case rr:l=s/Zn;break;case nr:l=s/Xn;break;case tr:l=s/Yn;break;default:l=s;break}return n?l:W.a(l)},t.daysInMonth=function(){return this.endOf(ar).$D},t.$locale=function(){return gr[this.$L]},t.locale=function(e,t){if(!e)return this.$L;var n=this.clone(),r=yr(e,t,!0);return r&&(n.$L=r),n},t.clone=function(){return W.w(this.$d,this)},t.toDate=function(){return new Date(this.valueOf())},t.toJSON=function(){return this.isValid()?this.toISOString():null},t.toISOString=function(){return this.$d.toISOString()},t.toString=function(){return this.$d.toUTCString()},e}(),Cr=Sr.prototype;U.prototype=Cr,[[`$ms`,er],[`$s`,tr],[`$m`,nr],[`$H`,rr],[`$W`,`day`],[`$M`,ar],[`$y`,sr],[`$D`,cr]].forEach(function(e){Cr[e[1]]=function(t){return this.$g(t,e[0],e[1])}}),U.extend=function(e,t){return e.$i||(e(t,Sr,U),e.$i=!0),U},U.locale=yr,U.isDayjs=vr,U.unix=function(e){return U(e*1e3)},U.en=gr[hr],U.Ls=gr,U.p={};var wr=function(e){return e.replace(/(\[[^\]]+])|(MMMM|MM|DD|dddd)/g,function(e,t,n){return t||n.slice(1)})},Tr={LTS:`h:mm:ss A`,LT:`h:mm A`,L:`MM/DD/YYYY`,LL:`MMMM D, YYYY`,LLL:`MMMM D, YYYY h:mm A`,LLLL:`dddd, MMMM D, YYYY h:mm A`},Er=function(e,t){return e.replace(/(\[[^\]]+])|(LTS?|l{1,4}|L{1,4})/g,function(e,n,r){var i=r&&r.toUpperCase();return n||t[r]||Tr[r]||wr(t[i])})},Dr=/(\[[^[]*\])|([-_:/.,()\s]+)|(A|a|Q|YYYY|YY?|ww?|MM?M?M?|Do|DD?|hh?|HH?|mm?|ss?|S{1,3}|z|ZZ?)/g,Or=/\d/,kr=/\d\d/,Ar=/\d{3}/,jr=/\d{4}/,G=/\d\d?/,Mr=/[+-]?\d+/,Nr=/[+-]\d\d:?(\d\d)?|Z/,Pr=/\d*[^-_:/,()\s\d]+/,Fr={},Ir=function(e){return e=+e,e+(e>68?1900:2e3)};function Lr(e){if(!e||e===`Z`)return 0;var t=e.match(/([+-]|\d\d)/g),n=+(t[1]*60)+(+t[2]||0);return n===0?0:t[0]===`+`?-n:n}var K=function(e){return function(t){this[e]=+t}},Rr=[Nr,function(e){var t=this.zone||(this.zone={});t.offset=Lr(e)}],zr=function(e){var t=Fr[e];return t&&(t.indexOf?t:t.s.concat(t.f))},Br=function(e,t){var n,r=Fr.meridiem;if(!r)n=e===(t?`pm`:`PM`);else for(var i=1;i<=24;i+=1)if(e.indexOf(r(i,0,t))>-1){n=i>12;break}return n},Vr={A:[Pr,function(e){this.afternoon=Br(e,!1)}],a:[Pr,function(e){this.afternoon=Br(e,!0)}],Q:[Or,function(e){this.month=(e-1)*3+1}],S:[Or,function(e){this.milliseconds=e*100}],SS:[kr,function(e){this.milliseconds=e*10}],SSS:[Ar,function(e){this.milliseconds=+e}],s:[G,K(`seconds`)],ss:[G,K(`seconds`)],m:[G,K(`minutes`)],mm:[G,K(`minutes`)],H:[G,K(`hours`)],h:[G,K(`hours`)],HH:[G,K(`hours`)],hh:[G,K(`hours`)],D:[G,K(`day`)],DD:[kr,K(`day`)],Do:[Pr,function(e){var t=Fr.ordinal,n=e.match(/\d+/);if(this.day=n[0],t)for(var r=1;r<=31;r+=1)t(r).replace(/\[|\]/g,``)===e&&(this.day=r)}],w:[G,K(`week`)],ww:[kr,K(`week`)],M:[G,K(`month`)],MM:[kr,K(`month`)],MMM:[Pr,function(e){var t=zr(`months`),n=(zr(`monthsShort`)||t.map(function(e){return e.slice(0,3)})).indexOf(e)+1;if(n<1)throw Error();this.month=n%12||n}],MMMM:[Pr,function(e){var t=zr(`months`).indexOf(e)+1;if(t<1)throw Error();this.month=t%12||t}],Y:[Mr,K(`year`)],YY:[kr,function(e){this.year=Ir(e)}],YYYY:[jr,K(`year`)],Z:Rr,ZZ:Rr};function Hr(e){var t=e.afternoon;if(t!==void 0){var n=e.hours;t?n<12&&(e.hours+=12):n===12&&(e.hours=0),delete e.afternoon}}function Ur(e){e=Er(e,Fr&&Fr.formats);for(var t=e.match(Dr),n=t.length,r=0;r<n;r+=1){var i=t[r],a=Vr[i],o=a&&a[0],s=a&&a[1];s?t[r]={regex:o,parser:s}:t[r]=i.replace(/^\[|\]$/g,``)}return function(e){for(var r={},i=0,a=0;i<n;i+=1){var o=t[i];if(typeof o==`string`)a+=o.length;else{var s=o.regex,c=o.parser,l=e.slice(a),u=s.exec(l)[0];c.call(r,u),e=e.replace(u,``)}}return Hr(r),r}}var Wr=function(e,t,n,r){try{if([`x`,`X`].indexOf(t)>-1)return new Date((t===`X`?1e3:1)*e);var i=Ur(t)(e),a=i.year,o=i.month,s=i.day,c=i.hours,l=i.minutes,u=i.seconds,d=i.milliseconds,f=i.zone,p=i.week,m=new Date,h=s||(!a&&!o?m.getDate():1),g=a||m.getFullYear(),_=0;a&&!o||(_=o>0?o-1:m.getMonth());var v=c||0,ee=l||0,y=u||0,te=d||0;if(f)return new Date(Date.UTC(g,_,h,v,ee,y,te+f.offset*60*1e3));if(n)return new Date(Date.UTC(g,_,h,v,ee,y,te));var b=new Date(g,_,h,v,ee,y,te);return p&&(b=r(b).week(p).toDate()),b}catch{return new Date(``)}},Gr=(function(e,t,n){n.p.customParseFormat=!0,e&&e.parseTwoDigitYear&&(Ir=e.parseTwoDigitYear);var r=t.prototype,i=r.parse;r.parse=function(e){var t=e.date,r=e.utc,a=e.args;this.$u=r;var o=a[1];if(typeof o==`string`){var s=a[2]===!0,c=a[3]===!0,l=s||c,u=a[2];c&&(u=a[2]),Fr=this.$locale(),!s&&u&&(Fr=n.Ls[u]),this.$d=Wr(t,o,r,n),this.init(),u&&u!==!0&&(this.$L=this.locale(u).$L),l&&t!=this.format(o)&&(this.$d=new Date(``)),Fr={}}else if(o instanceof Array)for(var d=o.length,f=1;f<=d;f+=1){a[1]=o[f-1];var p=n.apply(this,a);if(p.isValid()){this.$d=p.$d,this.$L=p.$L,this.init();break}f===d&&(this.$d=new Date(``))}else i.call(this,e)}});function Kr(e,t){let n=t.prototype,r=n.parse;n.parse=function(e){let t=e.date,n=e.args[1];r.call(this,e);let i=this.year(),a=i>=1900&&i<2e3,o=typeof n==`string`&&n.includes(`YYYY`),s=Array.isArray(n)&&typeof n[0]==`string`&&n[0].includes(`YYYY`),c=o||s,l=typeof t==`string`&&!t.includes(`${i}`);a&&c&&l&&(this.$d.setFullYear(i-1900),this.init())}}U.extend(Gr),U.extend(Kr);var qr=180,Jr=40,Yr=10,Xr=125,Zr=30,Qr=`YYYY`,$r=`no data`,ei=0,ti=`item`,ni=4,ri={linear:e=>e,logarithmic:e=>Math.log1p(e)},ii=e`var(--histogramDateRangeSliderColor, #4B65FE)`,ai=e`var(--histogramDateRangeSelectedRangeColor, #DBE0FF)`,oi=e`var(--histogramDateRangeBarIncludedFill, #2C2C2C)`,si=e`var(--histogramDateRangeActivityIndicator, #2C2C2C)`,ci=e`var(--histogramDateRangeBarExcludedFill, #CCCCCC)`,li=e`var(--histogramDateRangeInputRowMargin, 0)`,ui=e`var(--histogramDateRangeInputBorder, 0.5px solid #2C2C2C)`,di=e`var(--histogramDateRangeInputWidth, 35px)`,fi=e`var(--histogramDateRangeInputFontSize, 1.2rem)`,pi=e`var(--histogramDateRangeInputFontFamily, sans-serif)`,mi=e`var(--histogramDateRangeTooltipBackgroundColor, #2C2C2C)`,hi=e`var(--histogramDateRangeTooltipTextColor, #FFFFFF)`,gi=e`var(--histogramDateRangeTooltipFontSize, 1.1rem)`,_i=e`var(--histogramDateRangeTooltipFontFamily, sans-serif)`,q=class extends c{constructor(){super(...arguments),this.width=qr,this.height=Jr,this.sliderWidth=Yr,this.tooltipWidth=Xr,this.tooltipHeight=Zr,this.updateDelay=ei,this.dateFormat=Qr,this.missingDataMessage=$r,this.minDate=``,this.maxDate=``,this.disabled=!1,this.bins=[],this.updateWhileFocused=!1,this.binSnapping=`none`,this.tooltipLabel=ti,this.barScaling=`logarithmic`,this._tooltipOffsetX=0,this._tooltipOffsetY=0,this._isDragging=!1,this._isLoading=!1,this._minSelectedDate=``,this._maxSelectedDate=``,this._minDateMS=0,this._maxDateMS=0,this._dragOffset=0,this._histWidth=0,this._binWidth=0,this._histData=[],this._previousDateRange=``,this._updateDeferredWhileFocused=!1,this.drag=e=>{e.preventDefault(),!this.disabled&&(this.setDragOffset(e),this._isDragging=!0,this.addListeners(),this.cancelPendingUpdateEvent())},this.drop=()=>{this._isDragging&&(this.removeListeners(),this.beginEmitUpdateProcess()),this._isDragging=!1},this.move=e=>{let t=this.getBoundingClientRect().x,n=e.clientX-t-this._dragOffset;this._currentSlider.id===`slider-min`?this.minSelectedDate=this.translatePositionToDate(this.validMinSliderX(n)):(this.maxSelectedDate=this.translatePositionToDate(this.validMaxSliderX(n)),this.getMSFromString(this.maxSelectedDate)>this._maxDateMS&&(this.maxSelectedDate=this.maxDate))}}updated(){this._tooltip&&!this._tooltipContent&&(this._tooltip.hidden=!0)}disconnectedCallback(){this.removeListeners(),super.disconnectedCallback()}willUpdate(e){(e.has(`bins`)||e.has(`minDate`)||e.has(`maxDate`)||e.has(`minSelectedDate`)||e.has(`maxSelectedDate`)||e.has(`width`)||e.has(`height`)||e.has(`binSnapping`)||e.has(`barScaling`))&&this.handleDataUpdate()}handleDataUpdate(){this.hasBinData&&(this._histWidth=this.width-this.sliderWidth*2,this._minDateMS=this.snapTimestamp(this.getMSFromString(this.minDate)),this._maxDateMS=this.snapTimestamp(this.getMSFromString(this.maxDate)+this.snapInterval)+this.snapEndOffset,this._binWidth=this._histWidth/this._numBins,this._histData=this.calculateHistData(),this.minSelectedDate=this.minSelectedDate?this.minSelectedDate:this.minDate,this.maxSelectedDate=this.maxSelectedDate?this.maxSelectedDate:this.maxDate)}snapToNextSecond(e){return Math.ceil(e/1e3)*1e3}snapToMonth(e){let t=U(e),n=t.date()<16?0:1;return t.add(n,`month`).date(1).hour(0).minute(0).second(0).millisecond(0).valueOf()}snapToYear(e){let t=U(e),n=t.month()<6?0:1;return t.add(n,`year`).month(0).date(1).hour(0).minute(0).second(0).millisecond(0).valueOf()}snapTimestamp(e){switch(this.binSnapping){case`year`:return this.snapToYear(e);case`month`:return this.snapToMonth(e);default:return this.snapToNextSecond(e)}}get barScalingFunction(){return typeof this.barScaling==`string`?ri[this.barScaling]:this.barScaling}calculateHistData(){let{bins:e,height:t,dateRangeMS:n,_numBins:r,_minDateMS:i}=this,a=Math.min(...this.bins),o=Math.max(...this.bins),s=t/(a===o?1:this.barScalingFunction(o)),c=n/r;return e.map((e,t)=>{let n=this.snapTimestamp(t*c+i),r=this.formatDate(n),a=this.snapTimestamp((t+1)*c+i)+this.snapEndOffset,o=this.formatDate(a),l=this.formatDate(n,this.tooltipDateFormat),u=this.formatDate(a,this.tooltipDateFormat),d=l===u?l:`${l} - ${u}`;return{value:e,height:Math.floor(this.barScalingFunction(e)*s),binStart:r,binEnd:o,tooltip:d}})}get hasBinData(){return this._numBins>0}get _numBins(){return!this.bins||!this.bins.length?0:this.bins.length}get histogramLeftEdgeX(){return this.sliderWidth}get histogramRightEdgeX(){return this.width-this.sliderWidth}get snapInterval(){switch(this.binSnapping){case`year`:return 31536e6;case`month`:return 2592e6;default:return 0}}get snapEndOffset(){return this.binSnapping!==`none`&&this._numBins>1?-1:0}get tooltipDateFormat(){return this._tooltipDateFormat??this.dateFormat}set tooltipDateFormat(e){this._tooltipDateFormat=e}get loading(){return this._isLoading}set loading(e){this.disabled=e,this._isLoading=e}get minSelectedDate(){return this.formatDate(this.getMSFromString(this._minSelectedDate))}set minSelectedDate(e){if(!this._minSelectedDate){this._minSelectedDate=e;return}let t=this.getMSFromString(e),n=!Number.isNaN(t),r=t<=this.getMSFromString(this.maxSelectedDate);n&&r&&(this._minSelectedDate=this.formatDate(t)),this.requestUpdate()}get maxSelectedDate(){return this.formatDate(this.getMSFromString(this._maxSelectedDate))}set maxSelectedDate(e){if(!this._maxSelectedDate){this._maxSelectedDate=e;return}let t=this.getMSFromString(e),n=!Number.isNaN(t),r=t>=this.getMSFromString(this.minSelectedDate);n&&r&&(this._maxSelectedDate=this.formatDate(t)),this.requestUpdate()}get minSliderX(){let e=this.translateDateToPosition(this.minSelectedDate);return this.validMinSliderX(e)}get maxSliderX(){let e=this.snapTimestamp(this.getMSFromString(this.maxSelectedDate)+this.snapInterval),t=this.translateDateToPosition(this.formatDate(e));return this.validMaxSliderX(t)}get dateRangeMS(){return this._maxDateMS-this._minDateMS}showTooltip(e){if(this._isDragging||this.disabled)return;let t=e.currentTarget,n=t.x.baseVal.value+this.sliderWidth/2,r=t.dataset,i=`${this.tooltipLabel}${r.numItems===`1`?``:`s`}`,o=Number(r.numItems).toLocaleString(),s=9+this.tooltipHeight,c=this.getBoundingClientRect(),l=c.x+n,u=c.y;this._tooltipOffsetX=l-2+(this._binWidth-this.sliderWidth-this.tooltipWidth)/2+window.scrollX,this._tooltipOffsetY=u-s+window.scrollY,this._tooltipContent=a`
      ${o} ${i}<br />
      ${r.tooltip}
    `,this._tooltip.hidden=!1,this._tooltip.showPopover?.()}hideTooltip(){this._tooltipContent=void 0,this._tooltip.hidden=!0,this._tooltip.hidePopover?.()}validMinSliderX(e){let t=Math.min(this.translateDateToPosition(this.maxSelectedDate),this.histogramRightEdgeX);return e=this.clamp(e,this.histogramLeftEdgeX,t),Number.isNaN(e)||t<this.histogramLeftEdgeX?this.histogramLeftEdgeX:e}validMaxSliderX(e){let t=Math.max(this.histogramLeftEdgeX,this.translateDateToPosition(this.minSelectedDate));return e=this.clamp(e,t,this.histogramRightEdgeX),Number.isNaN(e)||t>this.histogramRightEdgeX?this.histogramRightEdgeX:e}addListeners(){window.addEventListener(`pointermove`,this.move),window.addEventListener(`pointerup`,this.drop),window.addEventListener(`pointercancel`,this.drop)}removeListeners(){window.removeEventListener(`pointermove`,this.move),window.removeEventListener(`pointerup`,this.drop),window.removeEventListener(`pointercancel`,this.drop)}beginEmitUpdateProcess(){this.cancelPendingUpdateEvent(),this._updateDeferredWhileFocused=!1,this._emitUpdatedEventTimer=setTimeout(()=>{if(this._emitUpdatedEventTimer=void 0,this.currentDateRangeString===this._previousDateRange)return;this._previousDateRange=this.currentDateRangeString;let e={detail:{minDate:this.minSelectedDate,maxDate:this.maxSelectedDate},bubbles:!0,composed:!0};this.dispatchEvent(new CustomEvent(`histogramDateRangeUpdated`,e))},this.updateDelay)}cancelPendingUpdateEvent(){return this._emitUpdatedEventTimer===void 0?!1:(clearTimeout(this._emitUpdatedEventTimer),this._emitUpdatedEventTimer=void 0,!0)}setDragOffset(e){this._currentSlider=e.currentTarget;let t=this._currentSlider.id===`slider-min`?this.minSliderX:this.maxSliderX,n=this.getBoundingClientRect().x;this._dragOffset=e.clientX-n-t}translatePositionToDate(e){let t=this.snapToNextSecond((e-this.sliderWidth)*this.dateRangeMS/this._histWidth);return this.formatDate(this._minDateMS+t)}translateDateToPosition(e){let t=this.getMSFromString(e);return this.sliderWidth+(t-this._minDateMS)*this._histWidth/this.dateRangeMS}clamp(e,t,n){return Math.min(Math.max(e,t),n)}handleInputFocus(){this.updateWhileFocused||(this._updateDeferredWhileFocused=this.cancelPendingUpdateEvent())}handleMinDateInput(e){let t=e.currentTarget,n=t.value!==this.minSelectedDate;n&&(this.minSelectedDate=t.value),(n||this._updateDeferredWhileFocused)&&this.beginEmitUpdateProcess()}handleMaxDateInput(e){let t=e.currentTarget,n=t.value!==this.maxSelectedDate;n&&(this.maxSelectedDate=t.value),(n||this._updateDeferredWhileFocused)&&this.beginEmitUpdateProcess()}handleKeyUp(e){if(e.key===`Enter`){let t=e.currentTarget;t.blur(),t.id===`date-min`?this.handleMinDateInput(e):t.id===`date-max`&&this.handleMaxDateInput(e)}}get currentDateRangeString(){return`${this.minSelectedDate}:${this.maxSelectedDate}`}getMSFromString(e){let t=typeof e==`string`?e:String(e);if((t.split(/(\d+)/).length-1)/2==1){let e=new Date(0,0);return e.setFullYear(Number(t)),e.getTime()}return U(t,[this.dateFormat,Qr]).valueOf()}handleBarClick(e){let t=e.currentTarget.dataset,n=(this.getMSFromString(t.binStart)+this.getMSFromString(t.binEnd))/2;Math.abs(n-this.getMSFromString(this.minSelectedDate))<Math.abs(n-this.getMSFromString(this.maxSelectedDate))?this.minSelectedDate=t.binStart:this.maxSelectedDate=t.binEnd,this.beginEmitUpdateProcess()}get minSliderTemplate(){let e=ni,t=`
            M${this.minSliderX},0
            h-${this.sliderWidth-e}
            q-${e},0 -${e},${e}
            v${this.height-e*2}
            q0,${e} ${e},${e}
            h${this.sliderWidth-e}
          `;return this.generateSliderSVG(this.minSliderX,`slider-min`,t)}get maxSliderTemplate(){let e=ni,t=`
            M${this.maxSliderX},0
            h${this.sliderWidth-e}
            q${e},0 ${e},${e}
            v${this.height-e*2}
            q0,${e} -${e},${e}
            h-${this.sliderWidth-e}
          `;return this.generateSliderSVG(this.maxSliderX,`slider-max`,t)}generateSliderSVG(e,t,n){let r=t===`slider-min`?1:-1;return m`
    <svg
      id=${t}
      class=${g({slider:!0,draggable:!this.disabled,dragging:this._isDragging})}
      @pointerdown=${this.drag}
    >
      <path d="${n} z" fill="${ii}" />
      <rect
        x="${e-this.sliderWidth*r+this.sliderWidth*.4*r}"
        y="${this.height/3}"
        width="1"
        height="${this.height/3}"
        fill="white"
      />
      <rect
        x="${e-this.sliderWidth*r+this.sliderWidth*.6*r}"
        y="${this.height/3}"
        width="1"
        height="${this.height/3}"
        fill="white"
      />
    </svg>
    `}get selectedRangeTemplate(){return m`
      <rect
        x="${this.minSliderX}"
        y="0"
        width="${this.maxSliderX-this.minSliderX}"
        height="${this.height}"
        fill="${ai}"
      />`}get histogramTemplate(){let e=this._histWidth/this._numBins,t=e-1,n=this.sliderWidth;return this._histData.map(r=>{let{minSelectedDate:i,maxSelectedDate:a}=this,o=r.height,s=this.isBefore(r.binEnd,i),c=this.isAfter(r.binStart,a),l=s||c?ci:oi,u=`stroke-dasharray: 0 ${t} ${o} ${t} 0 ${o}`,d=m`
        <rect
          class="bar-pointer-target"
          x=${n}
          y="0"
          width=${t}
          height=${this.height}
          @pointerenter=${this.showTooltip}
          @pointerleave=${this.hideTooltip}
          @click=${this.handleBarClick}
          fill="transparent"
          data-num-items=${r.value}
          data-bin-start=${r.binStart}
          data-bin-end=${r.binEnd}
          data-tooltip=${r.tooltip}
        />
        <rect
          class="bar"
          style=${u}
          x=${n}
          y=${this.height-o}
          width=${t}
          height=${o}
          fill=${l}
        />`;return n+=e,d})}isBefore(e,t){return this.getMSFromString(e)<this.getMSFromString(t)}isAfter(e,t){return this.getMSFromString(e)>this.getMSFromString(t)}formatDate(e,t=this.dateFormat){if(Number.isNaN(e))return``;let n=U(e);return n.year()<1e3?n.year(199999).format(t).replace(/199999/g,n.year().toString()):n.format(t)}get minInputTemplate(){return a`
      <input
        id="date-min"
        placeholder=${this.dateFormat}
        type="text"
        @focus=${this.handleInputFocus}
        @blur=${this.handleMinDateInput}
        @keyup=${this.handleKeyUp}
        .value=${_(this.minSelectedDate)}
        ?disabled=${this.disabled}
      />
    `}get maxInputTemplate(){return a`
      <input
        id="date-max"
        placeholder=${this.dateFormat}
        type="text"
        @focus=${this.handleInputFocus}
        @blur=${this.handleMaxDateInput}
        @keyup=${this.handleKeyUp}
        .value=${_(this.maxSelectedDate)}
        ?disabled=${this.disabled}
      />
    `}get minLabelTemplate(){return a`<label for="date-min" class="sr-only">Minimum date:</label>`}get maxLabelTemplate(){return a`<label for="date-max" class="sr-only">Maximum date:</label>`}get tooltipTemplate(){return a`
      <div id="tooltip" style=${d({width:`${this.tooltipWidth}px`,height:`${this.tooltipHeight}px`,top:`${this._tooltipOffsetY}px`,left:`${this._tooltipOffsetX}px`})} popover>${this._tooltipContent}</div>
    `}get histogramAccessibilityTemplate(){let e=``;return this.minSelectedDate&&this.maxSelectedDate?e=` from ${this.minSelectedDate} to ${this.maxSelectedDate}`:this.minSelectedDate?e=` from ${this.minSelectedDate}`:this.maxSelectedDate&&(e=` up to ${this.maxSelectedDate}`),a`<title id="histogram-title">${`Filter results for dates${e}`}</title
      ><desc id="histogram-desc">${`This histogram shows the distribution of dates${e}`}</desc>`}get noDataTemplate(){return a`
      <div class="missing-data-message">${this.missingDataMessage}</div>
    `}get activityIndicatorTemplate(){return this.loading?a`
      <ia-status-indicator mode="loading" ?hideDots=${!0}>
      </ia-status-indicator>
    `:n}render(){return this.hasBinData?a`
      <div
        id="container"
        class="
          noselect
          ${this._isDragging?`dragging`:``}
        "
        style="width: ${this.width}px"
      >
        ${this.activityIndicatorTemplate} ${this.tooltipTemplate}
        <div
          class="inner-container
          ${this.disabled?`disabled`:``}"
        >
          <svg
            width="${this.width}"
            height="${this.height}"
            aria-labelledby="histogram-title histogram-desc"
            @pointerleave="${this.drop}"
          >
            ${this.histogramAccessibilityTemplate} ${this.selectedRangeTemplate}
            <svg id="histogram">${this.histogramTemplate}</svg>
            ${this.minSliderTemplate} ${this.maxSliderTemplate}
          </svg>
          <div id="inputs">
            ${this.minLabelTemplate} ${this.minInputTemplate}
            <div class="dash">-</div>
            ${this.maxLabelTemplate} ${this.maxInputTemplate}
            <slot name="inputs-right-side"></slot>
          </div>
        </div>
      </div>
    `:this.noDataTemplate}};q.styles=e`
    .missing-data-message {
      text-align: center;
    }
    #container {
      margin: 0;
      touch-action: none;
      position: relative;
    }
    .disabled {
      opacity: 0.3;
    }
    ia-status-indicator {
      position: absolute;
      left: calc(50% - 10px);
      top: 10px;
      --icon-width: 20px;
      --loading-ring-color--: ${si};
    }

    /* prevent selection from interfering with tooltip, especially on mobile */
    /* https://stackoverflow.com/a/4407335/1163042 */
    .noselect {
      -webkit-touch-callout: none; /* iOS Safari */
      -webkit-user-select: none; /* Safari */
      -moz-user-select: none; /* Old versions of Firefox */
      -ms-user-select: none; /* Internet Explorer/Edge */
      user-select: none; /* current Chrome, Edge, Opera and Firefox */
    }
    .bar,
    .bar-pointer-target {
      /* create a transparent border around the hist bars to prevent "gaps" and
      flickering when moving around between bars. this also helps with handling
      clicks on the bars, preventing users from being able to click in between
      bars */
      stroke: rgba(0, 0, 0, 0);
      /* ensure transparent stroke wide enough to cover gap between bars */
      stroke-width: 2px;
    }
    .bar {
      /* ensure the bar's pointer target receives events, not the bar itself */
      pointer-events: none;
    }
    .bar-pointer-target:hover + .bar {
      /* highlight currently hovered bar */
      fill-opacity: 0.7;
    }
    .disabled .bar-pointer-target:hover + .bar {
      /* ensure no visual hover interaction when disabled */
      fill-opacity: 1;
    }
    /****** histogram ********/
    #tooltip {
      position: absolute;
      background: ${mi};
      margin: 0;
      border: 0;
      color: ${hi};
      text-align: center;
      border-radius: 3px;
      padding: 2px;
      font-size: ${gi};
      font-family: ${_i};
      touch-action: none;
      pointer-events: none;
      overflow: visible;
    }
    #tooltip:after {
      content: '';
      position: absolute;
      margin-left: -5px;
      top: 100%;
      left: 50%;
      /* arrow */
      border: 5px solid ${hi};
      border-color: ${mi} transparent transparent
        transparent;
    }
    /****** slider ********/
    .slider {
      shape-rendering: crispEdges; /* So the slider doesn't get blurry if dragged between pixels */
    }
    .draggable:hover {
      cursor: grab;
    }
    .dragging {
      cursor: grabbing !important;
    }
    /****** inputs ********/
    #inputs {
      display: flex;
      justify-content: center;
      margin: ${li};
    }
    #inputs .dash {
      position: relative;
      bottom: -1px;
      align-self: center; /* Otherwise the dash sticks to the top while the inputs grow */
    }
    input {
      width: ${di};
      margin: 0 3px;
      border: ${ui};
      border-radius: 2px !important;
      text-align: center;
      font-size: ${fi};
      font-family: ${pi};
    }
    .sr-only {
      position: absolute !important;
      width: 1px !important;
      height: 1px !important;
      margin: 0 !important;
      padding: 0 !important;
      border: 0 !important;
      overflow: hidden !important;
      white-space: nowrap !important;
      clip: rect(1px, 1px, 1px, 1px) !important;
      -webkit-clip-path: inset(50%) !important;
      clip-path: inset(50%) !important;
    }
  `,C([t({type:Number})],q.prototype,`width`,void 0),C([t({type:Number})],q.prototype,`height`,void 0),C([t({type:Number})],q.prototype,`sliderWidth`,void 0),C([t({type:Number})],q.prototype,`tooltipWidth`,void 0),C([t({type:Number})],q.prototype,`tooltipHeight`,void 0),C([t({type:Number})],q.prototype,`updateDelay`,void 0),C([t({type:String})],q.prototype,`dateFormat`,void 0),C([t({type:String})],q.prototype,`missingDataMessage`,void 0),C([t({type:String})],q.prototype,`minDate`,void 0),C([t({type:String})],q.prototype,`maxDate`,void 0),C([t({type:Boolean})],q.prototype,`disabled`,void 0),C([t({type:Array})],q.prototype,`bins`,void 0),C([t({type:Boolean})],q.prototype,`updateWhileFocused`,void 0),C([t({type:String})],q.prototype,`binSnapping`,void 0),C([t({type:String})],q.prototype,`tooltipLabel`,void 0),C([t({type:String})],q.prototype,`barScaling`,void 0),C([s()],q.prototype,`_tooltipOffsetX`,void 0),C([s()],q.prototype,`_tooltipOffsetY`,void 0),C([s()],q.prototype,`_tooltipContent`,void 0),C([s()],q.prototype,`_tooltipDateFormat`,void 0),C([s()],q.prototype,`_isDragging`,void 0),C([s()],q.prototype,`_isLoading`,void 0),C([S(`#tooltip`)],q.prototype,`_tooltip`,void 0),C([t({type:String})],q.prototype,`tooltipDateFormat`,null),C([t({type:Boolean})],q.prototype,`loading`,null),C([t()],q.prototype,`minSelectedDate`,null),C([t()],q.prototype,`maxSelectedDate`,null),q=C([ie(`ia-histogram-date-range`)],q);var vi=a`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      d="m100.657 0v45l-18.604-17.604-18.59 18.338-8.463-8.463 18.59-18.338-17.933-18.933zm-100.657 99.734v-45l18.604 17.604 18.59-18.338 8.463 8.463-18.59 18.338 17.933 18.933z"
    />
  </svg>
`,J=class extends c{constructor(){super(...arguments),this.binSnapping=`none`,this.barScaling=`logarithmic`,this.boundEscapeListener=e=>{e.key===`Escape`&&this.closeModal()}}render(){return a`
      <div id="container">
        <ia-histogram-date-range
          id="date-picker"
          .minDate=${this.minDate}
          .maxDate=${this.maxDate}
          .minSelectedDate=${this.minSelectedDate??this.minDate}
          .maxSelectedDate=${this.maxSelectedDate??this.maxDate}
          dateFormat=${b(this.customDateFormat)}
          tooltipDateFormat=${b(this.customTooltipDateFormat)}
          tooltipLabel=${b(this.customTooltipLabel)}
          .binSnapping=${this.binSnapping}
          .barScaling=${this.barScaling??n}
          .updateDelay=${0}
          updateWhileFocused
          missingDataMessage="..."
          .width=${560}
          .height=${120}
          .bins=${this.buckets}
          @histogramDateRangeUpdated=${this.histogramDateRangeUpdated}
        >
          <button
            id="apply-btn"
            slot="inputs-right-side"
            @click=${this.applyBtnClicked}
          >
            ${o(`Apply date range`)}
          </button>
        </ia-histogram-date-range>
      </div>
    `}connectedCallback(){super.connectedCallback?.(),this.setupEscapeListener()}disconnectedCallback(){super.disconnectedCallback?.(),this.removeEscapeListener()}setupEscapeListener(){document.addEventListener(`keydown`,this.boundEscapeListener)}removeEscapeListener(){document.removeEventListener(`keydown`,this.boundEscapeListener)}histogramDateRangeUpdated(e){this.minSelectedDate=e.detail.minDate,this.maxSelectedDate=e.detail.maxDate}applyBtnClicked(){let e=new CustomEvent(`histogramDateRangeApplied`,{detail:{minDate:this.minSelectedDate,maxDate:this.maxSelectedDate}});this.dispatchEvent(e),this.closeModal(),this.analyticsHandler?.sendEvent({category:k.default,action:A.histogramChangedFromModal,label:window.location.href})}closeModal(){this.modalManager&&(this.modalManager.closeModal(),this.dispatchEvent(new CustomEvent(`modalClosed`)))}static get styles(){return e`
      #container {
        display: flex;
        justify-content: center;
        padding: 40px 10px 10px;
        overflow: hidden;
      }

      #date-picker {
        --histogramDateRangeInputWidth: 50px;
        --histogramDateRangeInputRowMargin: 5px 0 0 0;
      }

      #apply-btn {
        margin: 0 0 0 5px;
        padding: 8px 10px;
        border: 0;
        border-radius: 4px;
        background: var(--primaryButtonBGColor, #194880);
        color: white;
        cursor: pointer;
      }
    `}};C([t({type:String})],J.prototype,`minDate`,void 0),C([t({type:String})],J.prototype,`maxDate`,void 0),C([t({type:String})],J.prototype,`minSelectedDate`,void 0),C([t({type:String})],J.prototype,`maxSelectedDate`,void 0),C([t({type:Array})],J.prototype,`buckets`,void 0),C([t({type:String})],J.prototype,`customDateFormat`,void 0),C([t({type:String})],J.prototype,`customTooltipDateFormat`,void 0),C([t({type:String})],J.prototype,`customTooltipLabel`,void 0),C([t({type:String})],J.prototype,`binSnapping`,void 0),C([t({type:String})],J.prototype,`barScaling`,void 0),C([t({type:Object,attribute:!1})],J.prototype,`modalManager`,void 0),C([t({type:Object,attribute:!1})],J.prototype,`analyticsHandler`,void 0),J=C([x(`expanded-date-picker`),i()],J);var yi=m`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path 
      d="m20.1116715 50.0035012-.1116715-.1085359 43.1159942-46.61088155c2.401537-2.18938917 4.6902018-3.28408375 6.8659943-3.28408375s4.1642651.63837733 5.9654178 1.91513199c1.8011528 1.27675467 3.1520173 2.97248092 4.0525937 5.08717877l-39.4020173 42.99768924 39.4020173 42.9976892c-.9005764 2.1146979-2.2514409 3.8104241-4.0525937 5.0871788-1.8011527 1.2767547-3.7896253 1.915132-5.9654178 1.915132-2.1013449 0-4.3900096-1.0573489-6.8659943-3.1720468l-43.1159942-46.7194174z"
    />
    <title>Go left icon</title>
  </svg>
`,bi=m`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path 
      d="m79.8883285 50.0035012.1116715-.1085359-43.1159942-46.61088155c-2.401537-2.18938917-4.6902018-3.28408375-6.8659943-3.28408375s-4.1642651.63837733-5.9654178 1.91513199c-1.8011528 1.27675467-3.1520173 2.97248092-4.0525937 5.08717877l39.4020173 42.99768924-39.4020173 42.9976892c.9005764 2.1146979 2.2514409 3.8104241 4.0525937 5.0871788 1.8011527 1.2767547 3.7896253 1.915132 5.9654178 1.915132 2.1013449 0 4.3900096-1.0573489 6.8659943-3.1720468l43.1159942-46.7194174z"
    />
    <title>Go right icon</title>
  </svg>
`,xi=m`
<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><path d="m98 50.5704143c-.2830293-.471515-.671154-1.1088947-1.1643742-1.9121392s-1.6003642-2.3617474-3.3214321-4.6755089c-1.7210678-2.3137614-3.522258-4.5325939-5.4035703-6.6564975-1.8813124-2.1239037-4.2828993-4.473133-7.2047606-7.0476881-2.9218612-2.5745551-5.8895067-4.7933876-8.9029363-6.6564976-3.0134295-1.86311-6.4628491-3.4330878-10.3482587-4.7099336-3.8854095-1.2768458-7.7822651-1.9142256-11.6905667-1.9121443-3.9083017.0020914-7.8051573.6154781-11.6905668 1.8401652-3.8854096 1.2246871-7.3702078 2.8301329-10.4543947 4.8163375-3.0841869 1.9862045-6.0278997 4.1695691-8.8311384 6.5500937s-5.2048256 4.7652219-7.2047605 7.1540919c-1.99993501 2.38887-3.75430043 4.5722346-5.26309632 6.5500938s-2.63883199 3.583305-3.39010829 4.8163374l-1.13003609 1.8401602c.2830293.4715149.67115403 1.1088946 1.16437421 1.9121391.49322017.8032445 1.5878776 2.3617475 3.28397229 4.6755089s3.47439274 4.521119 5.3348942 6.6220728c1.8605014 2.1009538 4.2506422 4.4387083 7.1704224 7.0132633 2.9197801 2.5745551 5.8874256 4.7819127 8.9029363 6.6220729 3.0155106 1.8401601 6.4774168 3.398663 10.3857184 4.6755088 3.9083017 1.2768458 7.8176438 1.9142256 11.7280266 1.9121443 3.9103827-.0020914 7.7957922-.6154781 11.6562286-1.8401652s7.3337886-2.818658 10.4200566-4.7819127 6.0299808-4.1351444 8.8311384-6.515669 5.2152311-4.7652219 7.2422203-7.1540919 3.8052873-4.5607597 5.3348942-6.515669c1.5296068-1.9549093 2.6721295-3.5488802 3.427568-4.7819127zm-24.5142913 0c0 6.467683-2.3079374 12.0152859-6.9238123 16.6428087s-10.1495139 6.9412843-16.600917 6.9412843c-6.4992683 0-12.0453939-2.3137615-16.6383767-6.9412843s-6.8894742-10.1751257-6.8894742-16.6428087 2.2964914-12.003811 6.8894742-16.608384 10.1391084-6.9068595 16.6383767-6.9068595c6.4534842 0 11.9871232 2.3022865 16.600917 6.9068595s6.9217312 10.140701 6.9238123 16.608384zm-23.5247293-10.552755c2.8261308 0 5.2870289 1.0619518 7.3826944 3.1858555 2.0956655 2.1239036 3.1434982 4.5795368 3.1434982 7.3668995 0 2.8332624-1.0478327 5.2888956-3.1434982 7.3668995-2.0956655 2.078004-4.5565636 3.1170059-7.3826944 3.1170059-2.873996 0-5.3348941-1.0264838-7.3826944-3.0794516-2.0478002-2.0529677-3.0717003-4.5200758-3.0717003-7.4013243 0-2.8332624 1.0239001-5.3003705 3.0717003-7.4013243 2.0478003-2.1009538 4.5086984-3.1514307 7.3826944-3.1514307z" fill="#000"/></svg>
`,Si=m`
<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><path d="m97.5245976 14.5407294-15.0809624 14.6188347c3.3026825 2.8601369 6.4111526 6.0234269 9.3254105 9.48987 2.9142578 3.4664431 5.0023086 6.2183876 6.2641522 8.2558335l1.9668021 3.1268688c-.291855.4841879-.6920826 1.1386987-1.2006828 1.9635322s-1.6502683 2.4252247-3.4250041 4.8011737c-1.7747358 2.3759489-3.6202894 4.6426342-5.5366607 6.8000558-1.9163713 2.1574217-4.3810437 4.5580085-7.3940172 7.2017606-3.0129735 2.643752-6.0731589 4.9104373-9.180556 6.8000558-3.1073972 1.8896186-6.6643798 3.4900098-10.6709478 4.8011737-4.0065681 1.3111639-8.0249391 1.9656747-12.055113 1.9635322-6.7019347 0-13.2343359-1.6732336-19.5972037-5.019701l-17.1185824 16.6562806-10.27179318-10.6917703 14.93288898-14.5449211c-3.2533247-2.8601369-6.3371159-6.0116436-9.25137378-9.45452-2.91425785-3.4428764-5.02698749-6.1819664-6.33818892-8.2172698l-1.8927654-3.0529552c.29185498-.4841879.69208259-1.1386987 1.20068282-1.9635322.50860022-.8248335 1.65026824-2.437008 3.42500406-4.8365236 1.77473582-2.3995157 3.62028938-4.6908389 5.53666072-6.8739696 1.9163713-2.1831307 4.3810437-4.5955009 7.3940172-7.2371105s6.0731589-4.9200783 9.180556-6.8354059c3.1073972-1.9153277 6.6772558-3.5275022 10.7095757-4.8365237 4.03232-1.3090215 8.0635669-1.9635322 12.0937409-1.9635322 6.5560071 0 13.0637294 1.6968003 19.5231669 5.090401l17.1185824-16.5823669zm-46.478979 24.584323 10.7803934-10.473243c-3.5451796-1.891761-7.3092505-2.8376415-11.2922126-2.8376415-6.6547228 0-12.3609169 2.3641657-17.1185824 7.0924969-4.7576654 4.7283312-7.1375711 10.437893-7.1397171 17.1286852 0 3.8306553.8251341 7.3945787 2.4754024 10.6917703l10.9284669-10.5471566v-.1446137c0-2.9094127 1.0687043-5.4546132 3.2061128-7.6356015 2.1374086-2.1809883 4.6868477-3.2714825 7.6483174-3.2714825h.5086002zm-1.1652739 21.5988543-10.7803935 10.6178566c3.5945375 1.9388943 7.4068932 2.9083415 11.4370672 2.9083415 6.6547228 0 12.3491139-2.375949 17.0831734-7.1278469s7.1021623-10.4486051 7.1043083-17.0901215c0-4.0234736-.874492-7.6356015-2.6234759-10.836384l-10.7095757 10.473243v.363141c0 2.9586884-1.0687042 5.4803223-3.2061128 7.5649015-2.1374085 2.0845792-4.6868476 3.1268688-7.6483173 3.1268688z" fill="#000"/></svg>
`;function Ci(e,t){return e?t?`hidden`:`selected`:`none`}function wi({facetType:e,bucket:t,collectionTitles:i,omitHideButton:s=!1,onCheckboxClick:c}){let l=`${e}:${t.key}-show-only`,u=`${e}:${t.key}-negative`,d=t.extraNote?a`<span class="facet-note">${t.extraNote}</span>`:n,f=e===`collection`?a`<a href="/details/${t.key}">
          ${i?.get(t.key)??t.key}
        </a> `:a`${t.displayText??t.key} ${d}`,p=t.count>0?new Intl.NumberFormat(M()).format(t.count):``,m=t.state===`hidden`,h=t.state===`selected`,g=Ee[e],_=De[e],v=`${g}: ${t.displayText??t.key}`,ee=o(h?r`Show all ${_}`:r`Only show ${v}`),y=o(r`Hide ${v}`),te=o(r`Unhide ${v}`),b=m?te:y,x=o(r`${v}, ${new Intl.NumberFormat(M()).format(t.count)} results`),S=c?e=>c(e,!1):void 0,ne=c?e=>c(e,!0):void 0;return a`
    <div class="facet-row-container">
      <div class="facet-checkboxes">
        <input
          type="checkbox"
          .name=${e}
          .value=${t.key}
          @click=${S}
          .checked=${h}
          class="select-facet-checkbox"
          title=${ee}
          id=${l}
          data-testid=${l}
        />
        <div
          class="hide-facet-container"
          ?hidden=${s&&!m}
        >
          <input
            type="checkbox"
            id=${u}
            .name=${e}
            .value=${t.key}
            @click=${ne}
            .checked=${m}
            class="hide-facet-checkbox"
          />
          <label
            for=${u}
            class="hide-facet-icon${m?` active`:``}"
            title=${b}
            data-testid=${u}
          >
            <span class="sr-only">${b}</span>
            <span class="eye eye-open">${xi}</span>
            <span class="eye eye-closed">${Si}</span>
          </label>
        </div>
      </div>
      <label
        for=${l}
        class="facet-info-display"
        title=${ee}
        aria-label=${x}
      >
        <div class="facet-title">${f}</div>
        <div class="facet-count">${p}</div>
      </label>
    </div>
  `}var Ti=e`var(--facet-row-border-top, 1px solid transparent)`,Ei=e`var(--facet-row-border-bottom, 1px solid transparent)`,Di=e`15px`,Oi=e`
  .facet-checkboxes {
    margin: 0 5px 0 0;
    display: flex;
    height: ${Di};
  }
  .facet-checkboxes input:first-child {
    margin-right: 5px;
  }
  .facet-checkboxes input {
    height: ${Di};
    width: ${Di};
    margin: 0;
  }
  .facet-row-container {
    display: flex;
    font-weight: 500;
    font-size: 1.2rem;
    margin: 0 auto;
    padding: 0.25rem 0;
    height: auto;
    border-top: ${Ti};
    border-bottom: ${Ei};
  }
  .facet-info-display {
    display: flex;
    flex: 1 1 0%;
    cursor: pointer;
    flex-wrap: wrap;
  }
  .facet-title {
    word-break: break-word;
    display: inline-block;
    flex: 1 1 0%;
  }
  .facet-note {
    color: #bbb;
  }
  .facet-count {
    text-align: right;
  }
  .select-facet-checkbox {
    cursor: pointer;
    display: inline-block;
  }
  .hide-facet-checkbox {
    position: absolute;
    clip: rect(0, 0, 0, 0);
    pointer-events: none;
  }
  .hide-facet-checkbox:focus-visible + .hide-facet-icon {
    outline-style: auto;
    outline-offset: 2px;
  }
  .hide-facet-icon {
    width: ${Di};
    height: ${Di};
    cursor: pointer;
    display: flex;
  }
  .eye {
    width: ${Di};
    height: ${Di};
    opacity: 0.3;
  }
  .hide-facet-icon:hover .eye,
  .active .eye {
    opacity: 1;
  }
  .hide-facet-icon:hover .eye-open,
  .hide-facet-icon .eye-closed {
    display: none;
  }
  .hide-facet-icon:hover .eye-closed,
  .hide-facet-icon.active .eye-closed {
    display: inline;
  }
  .hide-facet-icon.active .eye-open {
    display: none;
  }
  .sorting-icon {
    cursor: pointer;
  }

  a:link,
  a:visited {
    text-decoration: none;
    color: var(--ia-theme-link-color, #4b64ff);
  }
  a:hover {
    text-decoration: underline;
  }
`,ki=`onscrollend`in window,Ai=class{constructor(e,t){this.host=e,this.cfg=t,this.first=0,this.last=-1,this.colsPerPage=1,this.currentPage=0,this.viewportW=0,this.frame=0,this.landFrame=0,this.endTimer=0,this.relandAfterUpdate=!1,this.attach=e=>{let t=e instanceof HTMLElement?e:void 0;t!==this.scroller&&(this.detach(),t&&(this.scroller=t,t.addEventListener(`scroll`,this.onScroll,{passive:!0}),ki?t.addEventListener(`scrollend`,this.onScrollEnd):t.addEventListener(`scroll`,this.onScrollIdle,{passive:!0}),this.ro=new ResizeObserver(([e])=>this.onResize(e.contentRect.width)),this.ro.observe(t)))},this.onScroll=()=>{this.frame||(this.frame=requestAnimationFrame(()=>{this.frame=0,this.recompute()}))},this.onScrollIdle=()=>{clearTimeout(this.endTimer),this.endTimer=window.setTimeout(this.onScrollEnd,120)},this.onScrollEnd=()=>{this.land()},e.addController(this)}get pageWidth(){return this.cfg.colWidth*this.colsPerPage}get pageCount(){return Math.ceil(this.cfg.total/this.colsPerPage)}get totalWidth(){return this.cfg.colWidth*this.cfg.total}pageSpan(e){return Math.min(this.pageWidth,this.totalWidth-e*this.pageWidth)}setTotal(e){e!==this.cfg.total&&(this.cfg.total=e,this.recompute(!0),this.relandAfterUpdate=!0)}scrollToPage(e,t=`auto`){let n=Math.max(0,Math.min(e,this.pageCount-1));this.scroller?.scrollTo({left:n*this.pageWidth,behavior:t})}hostUpdated(){let e=this.scroller;if(e){if(this.realignTo!==void 0){let t=this.realignTo*this.pageWidth;e.scrollLeft===t&&(e.scrollLeft=t+1),e.scrollLeft=t,this.realignTo=void 0,this.relandAfterUpdate=!0}this.relandAfterUpdate&&!this.landFrame&&(this.relandAfterUpdate=!1,this.landFrame=requestAnimationFrame(()=>{this.landFrame=0,this.land()}))}}hostDisconnected(){this.detach()}detach(){let e=this.scroller;e&&(e.removeEventListener(`scroll`,this.onScroll),e.removeEventListener(`scroll`,this.onScrollIdle),e.removeEventListener(`scrollend`,this.onScrollEnd),this.ro?.disconnect(),this.ro=void 0,this.frame&&cancelAnimationFrame(this.frame),this.landFrame&&cancelAnimationFrame(this.landFrame),this.frame=0,this.landFrame=0,clearTimeout(this.endTimer),this.scroller=void 0)}onResize(e){let t=this.scroller;if(!t)return;this.viewportW=e;let n=Math.max(1,Math.floor(e/this.cfg.colWidth));if(n===this.colsPerPage){this.recompute();return}let r=Math.round(t.scrollLeft/this.cfg.colWidth);this.colsPerPage=n,this.realignTo=Math.floor(r/n),this.recompute(!0)}recompute(e=!1){let t=this.scroller;if(!t)return;let n=this.pageWidth,r=this.cfg.buffer??1,i=t.scrollLeft,a=Math.max(0,Math.floor(i/n)-r),o=Math.min(this.pageCount-1,Math.floor((i+this.viewportW)/n)+r);!e&&a===this.first&&o===this.last||(this.first=a,this.last=o,this.host.requestUpdate())}land(){let e=this.scroller;if(!e)return;let t=e.scrollLeft,n=t>=this.totalWidth-this.viewportW-1?this.pageCount-1:Math.round(t/this.pageWidth),r=Math.max(0,Math.min(n,this.pageCount-1));r!==this.currentPage&&(this.currentPage=r,this.host.requestUpdate()),this.cfg.onRest?.(r)}},ji=1e4,Mi=12,Ni=class extends c{constructor(){super(...arguments),this.buckets=[],this.rowsPerColumn=12,this.omitHideButtons=!1,this.win=new Ai(this,{colWidth:250,total:0,onRest:e=>this.rested(e)}),this.reportedPage=0}get currentPage(){return this.win.currentPage}get pageCount(){return this.win.pageCount}scrollToPage(e,t=`auto`){this.win.scrollToPage(e,t)}willUpdate(e){(e.has(`buckets`)||e.has(`rowsPerColumn`))&&this.win.setTotal(Math.ceil(this.buckets.length/this.rowsPerColumn)),this.keepFocusOnScreen()}render(){let e=this.win,t=[];for(let n=e.first;n<=e.last;n++)t.push(n);return a`
      <div class="scroll-nav">
        ${this.arrowTemplate(`prev`)}
        <div class="frame">
          <div
            class="scroller"
            role="group"
            aria-label=${b(this.label)}
            tabindex=${e.pageCount>0?0:-1}
            style="--rowsPerColumn: ${this.rowsPerColumn}"
            ${f(e.attach)}
            @click=${this.rowClicked}
            @focusin=${this.rowFocused}
          >
            <div class="sizer" style="width:${e.totalWidth}px">
              ${ee([e.pageCount,e.pageWidth,e.totalWidth],()=>this.snapStubsTemplate())}
              ${v(t,e=>e,t=>a`<div
                    class="page"
                    data-page=${t}
                    style="left:${t*e.pageWidth}px;width:${e.pageSpan(t)}px"
                  >
                    ${this.pageTemplate(t)}
                  </div>`)}
            </div>
          </div>
        </div>
        ${this.arrowTemplate(`next`)}
      </div>
    `}snapStubsTemplate(){let e=this.win;return a`${v(Array.from({length:e.pageCount},(e,t)=>t),e=>e,t=>a`<div
          class="snap-stub"
          style="left:${t*e.pageWidth}px;width:${e.pageSpan(t)}px"
        ></div>`)}`}pageTemplate(e){let{facetType:t,buckets:n,rowsPerColumn:r}=this;if(!t)return[];let{colsPerPage:i}=this.win,o=e*i,s=Math.min(o+i,Math.ceil(n.length/r)),c=[];for(let e=o;e<s;e++){let i=e*r,o=n.slice(i,i+r);c.push(a`<div class="column">
          ${o.map(e=>wi({facetType:t,bucket:e,collectionTitles:this.collectionTitles,omitHideButton:this.omitHideButtons}))}
        </div>`)}return c}arrowTemplate(e){let{currentPage:t,pageCount:n}=this.win,r=e===`prev`,i=r?t<=0:t>=n-1;return a`<button
      type="button"
      class=${g({"scroll-arrow":!0,[e]:!0,unneeded:n<=1})}
      ?disabled=${i}
      aria-label=${o(r?`Show previous page of values`:`Show next page of values`)}
      @click=${()=>this.arrowClicked(r?-1:1)}
    >
      ${r?yi:bi}
    </button>`}rowClicked(e){let t=e.target,{facetType:n}=this;if(!(t instanceof HTMLInputElement)||!n)return;let r=this.buckets.find(e=>e.key===t.value);if(!r)return;let i=t.classList.contains(`hide-facet-checkbox`);this.dispatchEvent(new CustomEvent(`facetClick`,{detail:{facetType:n,bucket:{...r,state:Ci(t.checked,i)},negative:i}}))}rowFocused(e){let{scroller:t}=this,n=e.target;if(!t||!(n instanceof HTMLElement)||n===t)return;let r=e.relatedTarget;if(r instanceof Node&&r!==t&&t.contains(r))return;let i=Number(n.closest(`.page`)?.dataset.page),{currentPage:a}=this.win;if(Number.isNaN(i)||i===a)return;let o=[...t.querySelectorAll(`.page[data-page="${a}"] input`)].filter(e=>!e.closest(`[hidden]`));(i<a?o[0]:o[o.length-1])?.focus({preventScroll:!0}),this.win.scrollToPage(a),requestAnimationFrame(()=>this.win.scrollToPage(a))}keepFocusOnScreen(){let e=this.shadowRoot?.activeElement?.closest(`.page`);if(!e)return;let t=Number(e.dataset.page);t>=this.win.first&&t<=this.win.last||this.scroller?.focus({preventScroll:!0})}arrowClicked(e){let t=this.pendingPage??this.win.currentPage,n=Math.max(0,Math.min(t+e,this.win.pageCount-1));this.pendingPage=n;let r=window.matchMedia?.(`(prefers-reduced-motion: reduce)`).matches;this.win.scrollToPage(n,r?`auto`:`smooth`)}rested(e){this.pendingPage=void 0,e!==this.reportedPage&&(this.reportedPage=e,this.dispatchEvent(new CustomEvent(`pageChanged`,{detail:e})))}static get styles(){let t=p(`250px`),n=p(`${Mi}px`);return[j,Oi,e`
      :host {
        display: block;
        --colWidth: ${t};
        --facetRowHeight: 2.4rem;
      }

      .scroll-nav {
        display: flex;
        align-items: center;
      }

      .frame {
        flex: 1 1 auto;
        min-width: 0;
        container-type: inline-size;
      }

      .scroller {
        /* Fallback where round() is unsupported: fill the frame */
        width: 100%;
        /* Otherwise round down to whole columns, so none is ever cut off */
        width: max(
          round(down, 100cqi, var(--colWidth)),
          min(100cqi, var(--colWidth))
        );
        height: calc(
          var(--rowsPerColumn) * var(--facetRowHeight) + ${n}
        );
        margin: 0 auto;
        outline-offset: 2px;
        overflow-x: auto;
        overflow-y: hidden;
        contain: strict;
        scroll-snap-type: x mandatory;
        /* scroll-snap-stop stays at its default 'normal' so a fling can cross
           many pages before resting. 'always' would force one page per gesture. */
      }

      /* Keep the scrollbar visible, so it's clear there is more to see */
      .scroller::-webkit-scrollbar {
        height: ${n};
      }
      .scroller::-webkit-scrollbar-track {
        background: #f1f1f1;
        border-radius: 6px;
      }
      .scroller::-webkit-scrollbar-thumb {
        background: #888;
        border-radius: 6px;
      }
      .scroller::-webkit-scrollbar-thumb:hover {
        background: #555;
      }
      /* Chrome ignores the rules above once these are set, so only Firefox gets them */
      @supports not selector(::-webkit-scrollbar) {
        .scroller {
          scrollbar-width: thin;
          scrollbar-color: #888 #f1f1f1;
        }
      }

      .sizer {
        position: relative;
        height: 100%;
      }

      .snap-stub {
        position: absolute;
        top: 0;
        height: 100%;
        scroll-snap-align: start;
        pointer-events: none;
        contain: strict;
      }

      .page {
        position: absolute;
        top: 0;
        height: 100%;
        display: flex;
        contain: content;
        content-visibility: auto;
        contain-intrinsic-size: auto none;
      }

      .column {
        flex: none;
        width: var(--colWidth);
        box-sizing: border-box;
        padding: 0 0.75rem;
      }

      /* Rows are a fixed height with one line of text, so every column holds
         exactly rowsPerColumn rows. The full text is in the row's tooltip. */
      .facet-row-container {
        height: var(--facetRowHeight);
        box-sizing: border-box;
        align-items: center;
      }
      .facet-info-display {
        flex-wrap: nowrap;
        min-width: 0;
      }
      .facet-title {
        min-width: 0;
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
        word-break: normal;
      }
      .facet-count {
        flex: none;
        padding-left: 0.5rem;
      }

      .scroll-arrow {
        flex: none;
        width: 2.4rem;
        padding: 0.5rem;
        background: none;
        border: none;
        cursor: pointer;
      }
      .scroll-arrow svg {
        height: 14px;
        fill: #2c2c2c;
      }
      .scroll-arrow:disabled {
        opacity: 0.3;
        cursor: default;
      }
      .scroll-arrow.unneeded {
        visibility: hidden;
      }

      /* Touch screens swipe; the space is better spent on the column */
      @media (max-width: 560px) {
        .scroll-arrow {
          display: none;
        }
      }
    `]}};C([t({type:String})],Ni.prototype,`facetType`,void 0),C([t({type:Array})],Ni.prototype,`buckets`,void 0),C([t({type:Object})],Ni.prototype,`collectionTitles`,void 0),C([t({type:Number})],Ni.prototype,`rowsPerColumn`,void 0),C([t({type:Boolean})],Ni.prototype,`omitHideButtons`,void 0),C([t({type:String})],Ni.prototype,`label`,void 0),C([S(`.scroller`)],Ni.prototype,`scroller`,void 0),Ni=C([x(`more-facets-scroller`),i()],Ni);var Pi=class extends c{constructor(){super(...arguments),this.leftValue=``,this.rightValue=``,this.side=`left`}render(){return a`
      <div id="container">
        <label for="switch-left">${this.leftLabel??this.leftValue}</label>
        <input
          type="radio"
          id="switch-left"
          class="sr-only"
          name="switch"
          .value=${this.leftValue}
          .checked=${this.side===`left`}
          @change=${this.handleRadioChange}
        />
        <input
          type="radio"
          id="switch-right"
          class="sr-only"
          name="switch"
          .value=${this.rightValue}
          .checked=${this.side===`right`}
          @change=${this.handleRadioChange}
        />
        <button
          id="switch-button"
          class=${this.side}
          aria-hidden="true"
          tabindex="-1"
          @click=${this.handleClick}
        >
          <div id="knob"></div>
        </button>
        <label for="switch-right">${this.rightLabel??this.rightValue}</label>
      </div>
    `}get value(){return this.side===`left`?this.leftValue:this.rightValue}get selectedLabel(){return this.side===`left`?this.leftLabel??this.leftValue:this.rightLabel??this.rightValue}handleClick(){this.side=this.side===`left`?`right`:`left`,this.emitChangeEvent()}handleRadioChange(){this.side=this.leftRadio.checked?`left`:`right`,this.emitChangeEvent()}emitChangeEvent(){let e=new CustomEvent(`change`,{detail:this.value});this.dispatchEvent(e)}static get styles(){let t=e`var(--switchWidth, 30px)`,n=e`var(--switchHeight, 14px)`,r=e`var(--switchMarginLeft, 5px)`,i=e`var(--switchMarginRight, 5px)`,a=e`var(--switchBorderWidth, 3px)`,o=e`var(--switchBgColor, #194880)`,s=e`var(--switchBorderColor, #194880)`;return[j,e`
        #container {
          display: inline-flex;
          align-items: center;
          flex-wrap: nowrap;
          font-size: ${e`var(--labelFontSize, 1.3rem)`};
        }

        #switch-button {
          width: ${t};
          height: ${n};
          margin: 0 ${i} 0 ${r};
          padding: 0;
          border: ${a} solid ${s};
          border-radius: ${n};
          box-sizing: content-box;
          background: ${o};
          appearance: none;
          cursor: pointer;
        }

        #switch-button.left #knob {
          transform: translateX(0);
        }

        #switch-button.right #knob {
          transform: translateX(calc(${t} - ${n}));
        }

        #switch-button:focus-visible {
          outline: 2px solid black;
          outline-offset: 2px;
        }

        #knob {
          display: block;
          width: ${n};
          height: ${n};
          border-radius: 50%;
          background: ${e`var(--knobColor, white)`};
          transition: transform ${e`var(--knobTransitionDuration, 0.25s)`} ${e`var(--knobTransitionFn, ease)`};
        }

        @media (prefers-reduced-motion: reduce) {
          #knob {
            transition-duration: 0.001s !important; /* Imperceptibly fast */
          }
        }
      `]}};C([t({type:String,attribute:!0})],Pi.prototype,`leftValue`,void 0),C([t({type:String,attribute:!0})],Pi.prototype,`leftLabel`,void 0),C([t({type:String,attribute:!0})],Pi.prototype,`rightValue`,void 0),C([t({type:String,attribute:!0})],Pi.prototype,`rightLabel`,void 0),C([t({type:String,attribute:!0})],Pi.prototype,`side`,void 0),C([S(`#switch-left`)],Pi.prototype,`leftRadio`,void 0),Pi=C([x(`toggle-switch`)],Pi);var Fi=m`
<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><path d="m14 53 24 24 48-52" fill="none" stroke="currentColor" stroke-width="14" stroke-linecap="round" stroke-linejoin="round"/></svg>
`,Ii=m`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><path d="m50 0c27.6142375 0 50 22.3857625 50 50s-22.3857625 50-50 50-50-22.3857625-50-50 22.3857625-50 50-50zm23.8159475 26.1840525c-1.4033215-1.4033215-3.5816761-1.5592461-5.1572272-.4677738l-.5598841.4677738-18.0988362 18.0989475-18.0988362-18.0989475-.5598841-.4677738c-1.5755511-1.0914723-3.7539057-.9355477-5.1572272.4677738-1.5787367 1.5787367-1.5787367 4.1383746 0 5.7171113l18.0989475 18.0988362-18.0989475 18.0988362c-1.5787367 1.5787367-1.5787367 4.1383746 0 5.7171113 1.4033215 1.4033215 3.5816761 1.5592461 5.1572272.4677738l.5598841-.4677738 18.0988362-18.0989475 18.0988362 18.0989475.5598841.4677738c1.5755511 1.0914723 3.7539057.9355477 5.1572272-.4677738 1.5787367-1.5787367 1.5787367-4.1383746 0-5.7171113l-18.0989475-18.0988362 18.0989475-18.0988362c1.5787367-1.5787367 1.5787367-4.1383746 0-5.7171113z" fill-rule="evenodd"/></svg>
`,Li=/[\p{Pd}\p{Pc}/]/gu;function Ri(e){let t=e.normalize(`NFKD`).replace(/\p{M}/gu,``).toLocaleLowerCase();return{spaced:Bi(t.replace(Li,` `)),joined:Bi(t.replace(Li,``))}}function zi(e,t){return e.spaced.includes(t.spaced)||e.joined.includes(t.joined)}function Bi(e){return e.replace(/\p{P}/gu,``).replace(/\s+/g,` `).trim()}var Y=window.location&&(window.location.hostname===`localhost`||window.location.host.match(/^(www|cat)-[a-z0-9]+\.archive\.org$/)||window.location.host.match(/\.code\.archive\.org$/)||window.location.host.match(/\.dev\.archive\.org$/)||window.location.host.match(/^ia-petabox-/)||window.location.host.match(/^local\.archive\.org/)||window.location.host.match(/^internetarchive\.github\.io$/))?console.log.bind(console):()=>{},X=class extends c{constructor(){super(...arguments),this.facetsLoading=!0,this.sortedBy=T.COUNT,this.isTvSearch=!1,this.unappliedFacetChanges=O(),this.filterText=``,this.loadFailed=!1,this.filteredBuckets=[],this.filterKeys=new Map,this.chosenBuckets=[],this.escapeHandler=e=>{e.key===`Escape`&&this.modalManager?.closeModal()}}willUpdate(e){e.has(`aggregations`)&&this.filterKeys.clear();let t=e.has(`aggregations`)||e.has(`sortedBy`)||e.has(`selectedFacets`)||e.has(`unappliedFacetChanges`);t&&(this.facetGroup=this.mergedFacets,this.chosenBuckets=this.listChosenBuckets()),(t||e.has(`filterText`))&&(this.filteredBuckets=this.filterBuckets()),(e.has(`facetKey`)||e.has(`query`)||e.has(`searchType`)||e.has(`filterMap`))&&(this.facetsLoading=!0,this.loadFailed=!1,this.sortedBy=this.searchType===w.TV?ke[this.facetKey]:Oe[this.facetKey],this.updateSpecificFacets())}updated(e){if((e.has(`filterText`)||e.has(`sortedBy`))&&this.scroller?.scrollToPage(0),this.chicletFocusIndex!==void 0){let e=this.shadowRoot?.querySelectorAll(`.chiclet-remove`)??[],t=Math.min(this.chicletFocusIndex,e.length-1);this.chicletFocusIndex=void 0,(e[t]??this.filterInput)?.focus()}}firstUpdated(){this.setupEscapeListeners()}disconnectedCallback(){super.disconnectedCallback(),document.removeEventListener(`keydown`,this.escapeHandler)}setupEscapeListeners(){this.modalManager&&document.addEventListener(`keydown`,this.escapeHandler)}get isSearchResultsPage(){let e=this.pageSpecifierParams?.pageType;return e===void 0||e===`search_results`}async updateSpecificFacets(){if(!this.facetKey)return;let e=this.query?.trim();if(!e&&this.isSearchResultsPage)return;let t={simpleParams:[this.facetKey]},n=ji,r={...this.pageSpecifierParams,query:e||``,identifiers:this.identifiers,filters:this.filterMap,aggregations:t,aggregationsSize:n,rows:0};try{let e=await this.searchService?.search(r,this.searchType),t=e?.success?.response;if(!t){Y(`More facets search failed`,e?.error),this.loadFailed=!0,this.aggregations=void 0;return}let{collectionTitles:n}=t;if(n)for(let[e,t]of Object.entries(n))this.collectionTitles?.set(e,t);this.aggregations=t.aggregations}catch(e){Y(`More facets search threw`,e),this.loadFailed=!0,this.aggregations=void 0}finally{this.facetsLoading=!1}}get mergedFacets(){if(!this.facetKey||!this.selectedFacets)return;let{aggregationFacetGroup:e}=this;if(!e)return;let t=this.currentSelections,n=e.buckets.map(e=>{let n=t[e.key];return n?{...e,state:n.state}:e}),r=new Set(n.map(e=>e.key)),i=this.selectedFacets[this.facetKey]??{};for(let[e,a]of Object.entries(i))r.has(e)||this.insertInSortOrder(n,{displayText:this.displayTextFor(e),key:e,count:a.count,state:t[e]?.state??a.state});return this.facetKey===`creator`&&this.isTvSearch&&n.forEach(e=>{e.displayText=(e.displayText??e.key)?.toLocaleUpperCase();let t=this.tvChannelAliases?.get(e.displayText);t&&t!==e.displayText&&(e.extraNote=`(${t})`)}),{...e,buckets:n}}get currentSelections(){return this.facetKey?{...this.selectedFacets?.[this.facetKey],...this.unappliedFacetChanges[this.facetKey]}:{}}listChosenBuckets(){let e=new Map(this.facetGroup?.buckets.map(e=>[e.key,e]));return Object.entries(this.currentSelections).map(([t,n])=>e.get(t)??{...n,displayText:this.displayTextFor(t)}).filter(e=>e.state!==`none`)}insertInSortOrder(e,t){let n=e.findIndex(e=>this.compareForSort(t,e)<0);e.splice(n===-1?e.length:n,0,t)}compareForSort(e,t){switch(this.sortedBy){case T.ALPHABETICAL:return(e.displayText??e.key).localeCompare(t.displayText??t.key);case T.NUMERIC:return Number(t.key)-Number(e.key);default:return t.count-e.count}}displayTextFor(e){return(this.facetKey===`collection`?this.collectionTitles?.get(e):void 0)??e}get aggregationFacetGroup(){if(!this.aggregations||!this.facetKey)return;let e=this.aggregations[this.facetKey];if(!e)return;let t=Ee[this.facetKey],n=e.getSortedBuckets(this.sortedBy);this.facetKey===`collection`&&(n=n?.filter(e=>{let t=e?.key?.toString();return!Ne[t]&&!t?.startsWith(`fav-`)}));let r=n.map(e=>{let t=`${e.key}`;return{displayText:this.displayTextFor(t),key:t,count:e.doc_count,state:`none`}});return this.facetKey===`collection`&&this.sortedBy===T.ALPHABETICAL&&r.sort((e,t)=>(e.displayText??e.key).localeCompare(t.displayText??t.key)),{title:t,key:this.facetKey,buckets:r}}filterBuckets(){let e=this.facetGroup?.buckets??[],t=Ri(this.filterText);return t.spaced?e.filter(e=>zi(this.filterKeyFor(e),t)):e}filterKeyFor(e){let t=this.filterKeys.get(e.key);return t===void 0&&(t=Ri([e.displayText??e.key,e.extraNote].filter(Boolean).join(` `)),this.filterKeys.set(e.key,t)),t}get noValuesMatch(){return Ri(this.filterText).spaced!==``&&this.filteredBuckets.length===0}get moreFacetsTemplate(){let e=this.facetKey?o(r`${Ee[this.facetKey]} values`):void 0;return a`
      <more-facets-scroller
        .facetType=${this.facetKey}
        .buckets=${this.filteredBuckets}
        .collectionTitles=${this.collectionTitles}
        ?omitHideButtons=${this.facetGroup?.buckets.length===1}
        label=${b(e)}
        @facetClick=${this.facetClicked}
        @pageChanged=${this.pageChanged}
      ></more-facets-scroller>
      ${l(this.loadFailed,()=>this.loadErrorTemplate,()=>l(this.noValuesMatch,()=>this.noMatchesTemplate))}
    `}get chicletsTemplate(){return a`
      <ul class="chiclets" aria-label=${o(r`Selected ${this.facetKey?De[this.facetKey]:``}`)}>
        ${v(this.chosenBuckets,e=>e.key,(e,t)=>this.chicletTemplate(e,t))}
      </ul>
    `}chicletTemplate(e,t){let n=e.state===`hidden`,i=[e.displayText??e.key,e.extraNote].filter(Boolean).join(` `),s=o(n?r`Unhide ${i}`:r`Deselect ${i}`);return a`
      <li class="chiclet" title=${i}>
        <span class="chiclet-icon" aria-hidden="true"
          >${n?Si:Fi}</span
        >
        <span class="sr-only"
          >${o(n?`Hidden:`:`Selected:`)}</span
        >
        <span class="chiclet-text">${i}</span>
        <button
          type="button"
          class="chiclet-remove"
          aria-label=${s}
          @click=${()=>this.chicletRemoved(e,t)}
        >
          ${Ii}
        </button>
      </li>
    `}get noMatchesTemplate(){return a`<p class="facets-message no-matches" role="status">
      ${o(`No matching values found.`)}
    </p>`}get loadErrorTemplate(){return a`<p class="facets-message load-error" role="alert">
      ${o(`Sorry, these values couldn’t be loaded. Please try again later.`)}
    </p>`}get loaderTemplate(){return a`
      <ia-status-indicator
        class="facets-loader"
        mode="loading"
      ></ia-status-indicator>
    `}get footerTemplate(){return a`
      <div class="footer">
        <button class="btn btn-cancel" type="button" @click=${this.cancelClick}>
          ${o(`Cancel`)}
        </button>
        <button
          class="btn btn-submit"
          type="button"
          @click=${this.applySearchFacetsClicked}
        >
          ${o(`Apply filters`)}
        </button>
      </div>
    `}sortFacetAggregation(e){this.sortedBy=e,this.dispatchEvent(new CustomEvent(`sortedFacets`,{detail:this.sortedBy}))}get modalHeaderTemplate(){let e=(this.sortedBy??Oe[this.facetKey])===T.COUNT?`left`:`right`,t=this.facetKey?De[this.facetKey]:``,i=this.facetKey?Ee[this.facetKey]:``;return a`<span class="sr-only">${o(`More facets for:`)}</span>
      <span class="title">${i}</span>
      <span class="header-controls">
        <span class="sort-controls">
          <label class="sort-label">${o(`Sort by:`)}</label>
          ${this.facetKey?a`<toggle-switch
                class="sort-toggle"
                leftValue=${T.COUNT}
                leftLabel=${o(`Count`)}
                rightValue=${Ae[this.facetKey]}
                .rightLabel=${i}
                side=${e}
                @change=${e=>{this.sortFacetAggregation(Number(e.detail))}}
              ></toggle-switch>`:n}
        </span>
        <span class="filter-controls">
          <label class="filter-label">${o(`Filter by:`)}</label>
          <ia-clearable-text-input
            class="filter-input"
            .value=${this.filterText}
            .placeholder=${o(r`Search ${t}…`)}
            .screenReaderLabel=${o(r`Filter ${t}`)}
            .clearButtonScreenReaderLabel=${o(`Clear filter`)}
            @input=${this.filterTextChanged}
          ></ia-clearable-text-input>
        </span>
      </span>`}render(){return a`
      ${this.facetsLoading?this.loaderTemplate:a`
            <section id="more-facets">
              <div class="header-content">${this.modalHeaderTemplate}</div>
              ${l(this.chosenBuckets.length,()=>this.chicletsTemplate)}
              <div class="facets-content">${this.moreFacetsTemplate}</div>
              ${this.footerTemplate}
            </section>
          `}
    `}filterTextChanged(e){let t=e.target;this.filterText=t.value}facetClicked(e){this.facetKey&&(this.unappliedFacetChanges=He(this.unappliedFacetChanges,this.facetKey,e.detail.bucket))}chicletRemoved(e,t){this.facetKey&&(this.unappliedFacetChanges=He(this.unappliedFacetChanges,this.facetKey,{...e,state:`none`}),this.chicletFocusIndex=t)}pageChanged(e){this.analyticsHandler?.sendEvent({category:k.default,action:A.moreFacetsPageChange,label:`${e.detail+1}`})}applySearchFacetsClicked(){let e=We(this.selectedFacets,this.unappliedFacetChanges),t=new CustomEvent(`facetsChanged`,{detail:e,bubbles:!0,composed:!0});this.dispatchEvent(t),this.unappliedFacetChanges=O(),this.modalManager?.closeModal(),this.analyticsHandler?.sendEvent({category:k.default,action:`${A.applyMoreFacetsModal}`,label:`${this.facetKey}`})}cancelClick(){this.unappliedFacetChanges=O(),this.modalManager?.closeModal(),this.analyticsHandler?.sendEvent({category:k.default,action:A.closeMoreFacetsModal,label:`${this.facetKey}`})}static get styles(){let t=e`var(--primaryButtonBGColor, #194880)`;return[j,e`
        section#more-facets {
          padding: 10px;
        }

        .header-content .title {
          display: block;
          text-align: left;
          font-size: 1.8rem;
          padding: 0 10px;
          font-weight: bold;
        }

        .header-controls {
          display: flex;
          flex-wrap: wrap;
          align-items: center;
          gap: 8px 20px;
          padding: 5px 10px 0;
        }

        /* Each label stays on the same line as its control when these wrap */
        .sort-controls,
        .filter-controls {
          display: inline-flex;
          align-items: center;
          gap: 5px;
          white-space: nowrap;
        }

        .sort-label,
        .filter-label {
          font-size: 1.3rem;
        }

        .sort-toggle {
          font-weight: normal;
        }

        .filter-input {
          --input-height: 2.5rem;
          --input-font-size: 1.3rem;
          --input-border-radius: 4px;
          --input-padding: 4px 8px;
          --input-focused-border-color: ${t};
          width: 16rem;
        }

        .chiclets {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
          margin: 0;
          padding: 10px 10px 0;
          list-style: none;
          /* About three rows; any more scroll, so the values stay in view */
          max-height: 7.8rem;
          overflow-y: auto;
        }

        .chiclet {
          display: inline-flex;
          align-items: center;
          gap: 4px;
          max-width: 24rem;
          padding: 2px 3px 2px 6px;
          border: 1px solid ${t};
          border-radius: 5px;
          background: #fff;
          color: #2c2c2c;
          font-size: 1.2rem;
          line-height: normal;
        }

        .chiclet-icon {
          display: flex;
          flex: none;
        }

        .chiclet-icon svg {
          width: 12px;
          height: 12px;
        }

        .chiclet-text {
          overflow: hidden;
          white-space: nowrap;
          text-overflow: ellipsis;
        }

        .chiclet-remove {
          display: flex;
          flex: none;
          padding: 2px;
          border: none;
          border-radius: 50%;
          background: none;
          cursor: pointer;
        }

        .chiclet-remove svg {
          width: 10px;
          height: 10px;
          opacity: 0.55;
        }

        .chiclet-remove:hover svg,
        .chiclet-remove:focus-visible svg {
          opacity: 1;
        }

        .facets-content {
          position: relative;
          font-size: 1.2rem;
          padding: 10px 0;
        }

        .facets-message {
          position: absolute;
          inset: 0;
          display: flex;
          align-items: center;
          justify-content: center;
          margin: 0;
          font-size: 1.4rem;
          color: #666;
        }

        .facets-loader {
          --icon-width: 70px;
          margin-bottom: 20px;
          display: block;
          margin-left: auto;
          margin-right: auto;
        }
        .btn {
          border: none;
          padding: 10px;
          margin-bottom: 10px;
          width: auto;
          border-radius: 4px;
          cursor: pointer;
          font-family: inherit;
        }
        .btn-cancel {
          background-color: #2c2c2c;
          color: white;
        }
        .btn-submit {
          background-color: ${t};
          color: white;
        }
        .footer {
          text-align: center;
        }

        @media (max-width: 560px) {
          /* The filter gets a line of its own here, so let it fill that line */
          .filter-controls {
            flex: 1 1 auto;
          }
          .filter-input {
            flex: 1 1 auto;
            width: auto;
            min-width: 12rem;
            --input-font-size: 1.2rem;
          }
        }
      `]}};C([t({type:String})],X.prototype,`facetKey`,void 0),C([t({type:String})],X.prototype,`query`,void 0),C([t({type:Array})],X.prototype,`identifiers`,void 0),C([t({type:Object})],X.prototype,`filterMap`,void 0),C([t({type:Number})],X.prototype,`searchType`,void 0),C([t({type:Object})],X.prototype,`pageSpecifierParams`,void 0),C([t({type:Object})],X.prototype,`collectionTitles`,void 0),C([t({type:Object})],X.prototype,`tvChannelAliases`,void 0),C([t({type:Boolean})],X.prototype,`facetsLoading`,void 0),C([t({type:Object})],X.prototype,`selectedFacets`,void 0),C([t({type:Number})],X.prototype,`sortedBy`,void 0),C([t({type:Boolean})],X.prototype,`isTvSearch`,void 0),C([t({type:Object})],X.prototype,`modalManager`,void 0),C([t({type:Object})],X.prototype,`searchService`,void 0),C([t({type:Object,attribute:!1})],X.prototype,`analyticsHandler`,void 0),C([s()],X.prototype,`aggregations`,void 0),C([s()],X.prototype,`facetGroup`,void 0),C([s()],X.prototype,`unappliedFacetChanges`,void 0),C([s()],X.prototype,`filterText`,void 0),C([s()],X.prototype,`loadFailed`,void 0),C([S(`more-facets-scroller`)],X.prototype,`scroller`,void 0),C([S(`ia-clearable-text-input`)],X.prototype,`filterInput`,void 0),X=C([x(`more-facets-content`),i()],X);var Vi,Hi=Vi=class extends c{constructor(){super(...arguments),this.omitHideButton=!1}render(){return a`${this.facetRowTemplate}`}get facetRowTemplate(){let{bucket:e,facetType:t}=this;return!e||!t?n:wi({facetType:t,bucket:e,collectionTitles:this.collectionTitles,omitHideButton:this.omitHideButton,onCheckboxClick:(e,t)=>this.facetClicked(e,t)})}facetClicked(e,t){let{bucket:n,facetType:r}=this;if(!n||!r)return;let{checked:i}=e.target;this.bucket={...n,state:Vi.getFacetState(i,t)},this.dispatchFacetClickEvent({facetType:r,bucket:this.bucket,negative:t})}dispatchFacetClickEvent(e){let t=new CustomEvent(`facetClick`,{detail:e});this.dispatchEvent(t)}static getFacetState(e,t){return Ci(e,t)}static get styles(){return[j,Oi]}};C([t({type:String})],Hi.prototype,`facetType`,void 0),C([t({type:Object})],Hi.prototype,`bucket`,void 0),C([t({type:Boolean,reflect:!0})],Hi.prototype,`omitHideButton`,void 0),C([t({type:Object})],Hi.prototype,`collectionTitles`,void 0),Hi=Vi=C([x(`facet-row`),i()],Hi);var Ui=class extends c{facetClicked(e){this.dispatchFacetClickEvent(e.detail)}dispatchFacetClickEvent(e){let t=new CustomEvent(`facetClick`,{detail:e,composed:!0});this.dispatchEvent(t)}get facetsTemplate(){let{facetGroup:e}=this;if(!e)return n;let t=e.buckets,r=t.length===1;return a`
      <div class="facet-rows" data-testid="facets-on-${e.key}">
        ${v(t,t=>`${e.key}:${t.key}`,t=>a`<facet-row
              .facetType=${e.key}
              .bucket=${t}
              .collectionTitles=${this.collectionTitles}
              ?omitHideButton=${r}
              @facetClick=${this.facetClicked}
            ></facet-row>`)}
      </div>
    `}render(){return a`${this.facetsTemplate}`}static get styles(){return e`
      .facet-rows {
        column-count: ${e`var(--facetsColumnCount, 1)`};
        column-gap: ${e`var(--facetsColumnGap, 15px)`};
      }

      a:link,
      a:visited {
        text-decoration: none;
        color: var(--ia-theme-link-color, #4b64ff);
      }
      a:hover {
        text-decoration: underline;
      }
    `}};C([t({type:Object})],Ui.prototype,`facetGroup`,void 0),C([t({type:Object})],Ui.prototype,`collectionTitles`,void 0),Ui=C([x(`facets-template`)],Ui);var Wi=class extends c{render(){return a`
      <div id="row">
        <input type="checkbox" disabled />
        <div class="tombstone-line"></div>
        <div class="tombstone-line"></div>
      </div>
    `}static get styles(){return e`
      #row {
        display: grid;
        grid-template-columns: 15px 1fr 36px;
        grid-gap: 9px 6px;
        align-items: center;
        margin: 2.5px auto;
        border: 1px solid transparent;
      }

      .tombstone-line {
        background: #ddd;
        height: 6px;
        border-radius: 50px;
      }

      input[type='checkbox'] {
        width: 15px;
        height: 15px;
        margin: 0;
      }
    `}};Wi=C([x(`facet-tombstone-row`)],Wi);var Z=class extends c{constructor(){super(...arguments),this.moreLinksVisible=!0,this.facetsLoading=!1,this.histogramAggregationLoading=!1,this.collapsableFacets=!1,this.showHistogramDatePicker=!1,this.allowExpandingDatePicker=!1,this.suppressMediatypeFacets=!1,this.parentCollections=[],this.collectionPagePath=`/details/`,this.isManageView=!1,this.isTvSearch=!1,this.facetDisplayOrder=we,this.openFacets={subject:!1,lending:!1,mediatype:!1,language:!1,creator:!1,collection:!1,year:!1,clip_type:!1,program:!1,person:!1,sponsor:!1},this.allowedFacetCount=6,this.handleExpandedDatePickerClosed=()=>{this.modalManager?.classList.remove(`expanded-date-picker`)},this.histogramDateRangeUpdated=e=>{let{minDate:t,maxDate:n}=e.detail,r=new CustomEvent(`histogramDateRangeUpdated`,{detail:{minDate:t,maxDate:n}});this.dispatchEvent(r)}}render(){let e=g({loading:this.facetsLoading,managing:this.isManageView}),t=this.isTvSearch?o(`Date Published`):o(`Year Published`),r=`date-picker-label`;return a`
      <div id="container" class=${e}>
        ${this.showHistogramDatePicker&&(this.histogramAggregation||this.histogramAggregationLoading)?a`
              <section
                class="facet-group"
                aria-labelledby=${r}
                data-testid="facet-group-header-label-date-picker"
              >
                <h3 id=${r}>
                  ${t} <span class="sr-only">${o(`range filter`)}</span>
                  ${this.expandDatePickerBtnTemplate}
                </h3>
                ${this.histogramTemplate}
              </section>
            `:n}
        ${this.collectionPartOfTemplate}
        <slot name="facets-top"></slot>
        ${this.mergedFacets.map(e=>this.getFacetGroupTemplate(e))}
      </div>
    `}get collectionPartOfTemplate(){if(!this.parentCollections?.length)return n;let e=`partof-heading`;return a`
      <section
        class="facet-group partof-collections"
        aria-labelledby=${e}
        data-testid="facet-group-partof-collections"
      >
        <div class="facet-group-header">
          <h3 id=${e}>${o(`Part Of`)}</h3>
        </div>
        <ul>
          ${y(this.parentCollections,e=>a` <li>
              <a
                href=${`${this.baseNavigationUrl}${this.collectionPagePath}${e}`}
                data-id=${e}
                @click=${this.partOfCollectionClicked}
              >
                ${this.collectionTitles?.get(e)??e}
              </a>
            </li>`)}
        </ul>
      </section>
    `}partOfCollectionClicked(e){this.analyticsHandler?.sendEvent({category:k.default,action:A.partOfCollectionClicked,label:e.target.dataset.id})}get histogramProps(){let{histogramAggregation:e}=this;if(!e)return;let t=e.first_bucket_year??e.first_bucket_key,n=e.last_bucket_year??e.last_bucket_key;if(t==null||n==null)return;let r=e.first_bucket_month??1,i=e.last_bucket_month??12,a=e.interval??1,o=e.interval_in_months??12,s=e=>e.toString().padStart(2,`0`);if(this.isTvSearch){let a=o<12;return{buckets:e.buckets,dateFormat:`YYYY-MM`,tooltipDateFormat:a?`MMM YYYY`:`YYYY`,tooltipLabel:`broadcast`,binSnapping:a?`month`:`year`,barScaling:`linear`,minDate:`${t}-${s(r)}`,maxDate:`${n}-${s(i+o-1)}`}}return{buckets:e.buckets,dateFormat:`YYYY`,tooltipDateFormat:`YYYY`,tooltipLabel:`item`,binSnapping:`year`,barScaling:`logarithmic`,minDate:`${t}`,maxDate:`${n+a-1}`}}showDatePickerModal(){let{histogramProps:e}=this;if(!e)return;let{buckets:t,dateFormat:n,tooltipDateFormat:r,tooltipLabel:i,binSnapping:s,barScaling:c,minDate:l,maxDate:u}=e,d=a`
      <expanded-date-picker
        ${f(e=>{if(e&&e instanceof J){let t=e;t.minSelectedDate=this.minSelectedDate,t.maxSelectedDate=this.maxSelectedDate}})}
        .minDate=${l}
        .maxDate=${u}
        .minSelectedDate=${this.minSelectedDate}
        .maxSelectedDate=${this.maxSelectedDate}
        .customDateFormat=${n}
        .customTooltipDateFormat=${r}
        .customTooltipLabel=${i}
        .binSnapping=${s}
        .barScaling=${c}
        .buckets=${t}
        .modalManager=${this.modalManager}
        .analyticsHandler=${this.analyticsHandler}
        @histogramDateRangeApplied=${this.histogramDateRangeUpdated}
        @modalClosed=${this.handleExpandedDatePickerClosed}
      ></expanded-date-picker>
    `,p=new oe({bodyColor:`#fff`,headerColor:`#194880`,showHeaderLogo:!1,closeOnBackdropClick:!0,title:a`${o(`Select a date range`)}`});this.modalManager?.classList.add(`expanded-date-picker`),this.modalManager?.showModal({config:p,customModalContent:d,userClosedModalCallback:this.handleExpandedDatePickerClosed}),this.analyticsHandler?.sendEvent({category:k.default,action:A.histogramExpanded,label:window.location.href})}updated(e){e.has(`selectedFacets`)&&this.dispatchFacetsChangedEvent()}dispatchFacetsChangedEvent(){let e=new CustomEvent(`facetsChanged`,{detail:this.selectedFacets});this.dispatchEvent(e)}get expandDatePickerBtnTemplate(){return this.allowExpandingDatePicker&&!this.facetsLoading?a`<button
          class="expand-date-picker-btn"
          aria-haspopup="dialog"
          @click=${this.showDatePickerModal}
        >
          <span class="sr-only">${o(`Expand date histogram`)}</span>
          <span aria-hidden="true">${vi}</span>
        </button>`:n}get histogramTemplate(){if(this.histogramAggregationLoading)return a` <div class="histogram-loading-indicator">&hellip;</div> `;let{histogramProps:e}=this;if(!e)return n;let{buckets:t,dateFormat:r,tooltipDateFormat:i,tooltipLabel:o,binSnapping:s,barScaling:c,minDate:l,maxDate:u}=e;return a`
      <ia-histogram-date-range
        class=${this.isTvSearch?`wide-inputs`:``}
        .minDate=${l}
        .maxDate=${u}
        .minSelectedDate=${this.minSelectedDate??l}
        .maxSelectedDate=${this.maxSelectedDate??u}
        .updateDelay=${100}
        .dateFormat=${r}
        .tooltipDateFormat=${i}
        .tooltipLabel=${o}
        .binSnapping=${s}
        .barScaling=${c}
        .bins=${t}
        missingDataMessage="..."
        .width=${this.collapsableFacets&&this.contentWidth?this.contentWidth:180}
        @histogramDateRangeUpdated=${this.histogramDateRangeUpdated}
      ></ia-histogram-date-range>
    `}get mergedFacets(){let e=[];return this.facetDisplayOrder.forEach(t=>{if(t===`mediatype`&&this.suppressMediatypeFacets)return;let n=this.selectedFacetGroups.find(e=>e.key===t),r=this.aggregationFacetGroups.find(e=>e.key===t);if(n&&!r){e.push(n);return}if(!r)return;let i=n??r,a=n?.buckets.map(e=>{let t=r.buckets.find(t=>t.key===e.key);return t?{...e,count:t.count}:e})??[];r.buckets.forEach(e=>{a.find(t=>t.key===e.key)||a.push(e)});let o=Object.keys(n?.buckets||[])?.length;if(o<this.allowedFacetCount&&(o=this.allowedFacetCount),t===`lending`&&(a=a.filter(e=>je[e.key])),Ke(a,Oe[t]),t===`mediatype`){let e=a.findIndex(e=>e.key===`collection`);if(e>=o){let[t]=a.splice(e,1);o>this.allowedFacetCount&&(o+=1),a.splice(o-1,0,t)}}t===`creator`&&this.isTvSearch&&a.forEach(e=>{e.displayText=(e.displayText??e.key)?.toLocaleUpperCase();let t=this.tvChannelAliases?.get(e.displayText);t&&t!==e.displayText&&(e.extraNote=`(${t})`)}),t===`clip_type`&&a.forEach(e=>{e.displayText??(e.displayText=e.key),e.displayText=e.displayText.charAt(0).toLocaleUpperCase()+e.displayText.slice(1)}),i.buckets=a.slice(0,o),e.push(i)}),e}get selectedFacetGroups(){return this.selectedFacets?Object.entries(this.selectedFacets).map(([e,t])=>{let n=e;return{title:Ee[n],key:n,buckets:Object.entries(t).map(([e,t])=>{let r=e,i=Me[n];return i&&(r=i[e]??e),{displayText:r,key:e,count:t.count,state:t.state}})}}):[]}get aggregationFacetGroups(){let e=[];return Object.entries(this.aggregations??[]).forEach(([t,n])=>{if([`year_histogram`,`date_histogram`].includes(t))return;let r=t,i=Ee[r];if(!i)return;let a=n.getSortedBuckets(Oe[r]);r===`collection`&&(a=a?.filter(e=>{let t=e?.key?.toString();return!Ne[t]&&!t?.startsWith(`fav-`)}));let o={title:i,key:r,buckets:a.map(e=>{let t=e.key,n=`${e.key}`,i=Me[r];return i&&(n=i[e.key]??`${e.key}`),{displayText:n,key:`${t}`,count:e.doc_count,state:`none`}})};e.push(o)}),e}getFacetGroupTemplate(e){if(!this.facetsLoading&&e.buckets.length===0)return n;let{key:t}=e,r=this.openFacets[t],i=a`
      <span class="collapser ${r?`open`:``}"> ${Je} </span>
    `,s=()=>{let e={...this.openFacets};e[t]=!r,this.openFacets=e},c=`facet-group-header-label-${e.key}`;return a`
      <section
        class="facet-group ${this.collapsableFacets?`mobile`:``}"
        aria-labelledby=${c}
        data-testid=${c}
      >
        <div class="facet-group-header">
          <h3
            id=${c}
            @click=${s}
            @keyup=${s}
          >
            ${this.collapsableFacets?i:n} ${e.title}
            <span class="sr-only">${o(`filters`)}</span>
          </h3>
        </div>
        <div
          class="facet-group-content ${r?`open`:``}"
          data-testid="facet-group-content-${e.key}"
        >
          ${this.facetsLoading?this.getTombstoneFacetGroupTemplate():a`
                ${this.getFacetTemplate(e)}
                ${this.searchMoreFacetsLink(e)}
              `}
        </div>
      </section>
    `}getTombstoneFacetGroupTemplate(){return a`
      ${y([,,,,,].fill(null),()=>a`<facet-tombstone-row></facet-tombstone-row>`)}
    `}searchMoreFacetsLink(e){if(!this.moreLinksVisible||e.key===`lending`||Object.keys(e.buckets).length<this.allowedFacetCount)return n;let t=Oe[e.key];return a`<button
      class="more-link"
      @click=${()=>{this.showMoreFacetsModal(e,t),this.analyticsHandler?.sendEvent({category:k.default,action:A.showMoreFacetsModal,label:e.key}),this.dispatchEvent(new CustomEvent(`showMoreFacets`,{detail:e.key}))}}
      data-testid="more-link-btn"
    >
      ${o(`More...`)}
    </button>`}async showMoreFacetsModal(e,t){let n=a`
      <more-facets-content
        .analyticsHandler=${this.analyticsHandler}
        .facetKey=${e.key}
        .query=${this.query}
        .identifiers=${this.identifiers}
        .filterMap=${this.filterMap}
        .pageSpecifierParams=${this.pageSpecifierParams}
        .modalManager=${this.modalManager}
        .searchService=${this.searchService}
        .searchType=${this.searchType}
        .collectionTitles=${this.collectionTitles}
        .tvChannelAliases=${this.tvChannelAliases}
        .selectedFacets=${this.selectedFacets}
        .sortedBy=${t}
        .isTvSearch=${this.isTvSearch}
        @facetsChanged=${e=>{let t=new CustomEvent(`facetsChanged`,{detail:e.detail,bubbles:!0,composed:!0});this.dispatchEvent(t)}}
      >
      </more-facets-content>
    `,r=new oe({bodyColor:`#fff`,headerColor:`#194880`,showHeaderLogo:!1,closeOnBackdropClick:!0,title:a`${o(`Select filters`)}`});this.modalManager?.classList.add(`more-search-facets`),this.modalManager?.showModal({config:r,customModalContent:n,userClosedModalCallback:()=>{this.modalManager?.classList.remove(`more-search-facets`)}})}getFacetTemplate(e){return a`
      <facets-template
        .collectionPagePath=${this.collectionPagePath}
        .facetGroup=${e}
        .selectedFacets=${this.selectedFacets}
        .collectionTitles=${this.collectionTitles}
        @facetClick=${t=>{this.selectedFacets=He(this.selectedFacets,e.key,t.detail.bucket,!0);let n=new CustomEvent(`facetsChanged`,{detail:this.selectedFacets,bubbles:!0,composed:!0});this.dispatchEvent(n)}}
      ></facets-template>
    `}static get styles(){return[j,e`
        a:link {
          text-decoration: none;
          color: var(--ia-theme-link-color, #4b64ff);
        }
        a:link:hover {
          text-decoration: underline;
        }

        #container.loading {
          opacity: 0.5;
        }

        #container.managing {
          opacity: 0.3;
        }

        .histogram-loading-indicator {
          width: 100%;
          height: 5.25rem;
          margin-top: 1.75rem;
          font-size: 1.4rem;
          text-align: center;
        }

        .collapser {
          display: inline-block;
          cursor: pointer;
          width: 10px;
          height: 10px;
        }

        .collapser svg {
          transition: transform 0.2s ease-in-out;
        }

        .collapser.open svg {
          transform: rotate(90deg);
        }

        .facet-group {
          margin-bottom: 2rem;
        }

        .facet-group h3 {
          margin-bottom: 0.7rem;
        }

        .facet-group.mobile h3 {
          cursor: pointer;
        }

        .facet-group-header {
          display: flex;
          margin-bottom: 0.7rem;
          justify-content: space-between;
          border-bottom: 1px solid rgb(232, 232, 232);
        }

        .facet-group-content {
          transition: max-height 0.2s ease-in-out;
        }

        .facet-group.mobile .facet-group-content {
          max-height: 0;
          overflow: hidden;
        }

        .facet-group.mobile .facet-group-content.open {
          max-height: 2000px;
        }

        .partof-collections ul {
          list-style-type: none;
          padding: 0;
          font-size: 1.2rem;
        }

        h3 {
          font-size: 1.4rem;
          margin: 0;
        }

        .more-link {
          font-size: 1.2rem;
          text-decoration: none;
          padding: 0;
          margin-top: 0.25rem;
          background: inherit;
          border: 0;
          color: var(--ia-theme-link-color, #4b64ff);
          cursor: pointer;
        }

        #date-picker-label {
          display: flex;
          justify-content: space-between;
        }

        .expand-date-picker-btn {
          margin: 0;
          padding: 0;
          border: 0;
          appearance: none;
          background: none;
          cursor: pointer;
        }

        .expand-date-picker-btn svg {
          width: 14px;
          height: 14px;
        }

        .sorting-icon {
          height: 15px;
          cursor: pointer;
        }

        ia-histogram-date-range.wide-inputs {
          --histogramDateRangeInputWidth: 4.8rem;
        }
      `]}};C([t({type:Object})],Z.prototype,`searchService`,void 0),C([t({type:Number})],Z.prototype,`searchType`,void 0),C([t({type:Object})],Z.prototype,`aggregations`,void 0),C([t({type:Object})],Z.prototype,`histogramAggregation`,void 0),C([t({type:String})],Z.prototype,`minSelectedDate`,void 0),C([t({type:String})],Z.prototype,`maxSelectedDate`,void 0),C([t({type:Boolean})],Z.prototype,`moreLinksVisible`,void 0),C([t({type:Boolean})],Z.prototype,`facetsLoading`,void 0),C([t({type:Boolean})],Z.prototype,`histogramAggregationLoading`,void 0),C([t({type:Object})],Z.prototype,`selectedFacets`,void 0),C([t({type:Boolean})],Z.prototype,`collapsableFacets`,void 0),C([t({type:Number})],Z.prototype,`contentWidth`,void 0),C([t({type:Boolean})],Z.prototype,`showHistogramDatePicker`,void 0),C([t({type:Boolean})],Z.prototype,`allowExpandingDatePicker`,void 0),C([t({type:Boolean})],Z.prototype,`suppressMediatypeFacets`,void 0),C([t({type:String})],Z.prototype,`query`,void 0),C([t({type:Array})],Z.prototype,`identifiers`,void 0),C([t({type:Object})],Z.prototype,`pageSpecifierParams`,void 0),C([t({type:Array})],Z.prototype,`parentCollections`,void 0),C([t({type:Object})],Z.prototype,`filterMap`,void 0),C([t({type:String})],Z.prototype,`baseNavigationUrl`,void 0),C([t({type:String})],Z.prototype,`collectionPagePath`,void 0),C([t({type:Boolean})],Z.prototype,`isManageView`,void 0),C([t({type:Boolean})],Z.prototype,`isTvSearch`,void 0),C([t({type:Array})],Z.prototype,`facetDisplayOrder`,void 0),C([t({type:Object,attribute:!1})],Z.prototype,`modalManager`,void 0),C([t({type:Object,attribute:!1})],Z.prototype,`resizeObserver`,void 0),C([t({type:Object,attribute:!1})],Z.prototype,`featureFeedbackService`,void 0),C([t({type:Object,attribute:!1})],Z.prototype,`recaptchaManager`,void 0),C([t({type:Object,attribute:!1})],Z.prototype,`analyticsHandler`,void 0),C([t({type:Object,attribute:!1})],Z.prototype,`collectionTitles`,void 0),C([t({type:Object,attribute:!1})],Z.prototype,`tvChannelAliases`,void 0),C([s()],Z.prototype,`openFacets`,void 0),Z=C([x(`collection-facets`),i()],Z);var Gi=class extends c{render(){return a`
      <div class="lds-ring">
        <div></div>
        <div></div>
        <div></div>
        <div></div>
      </div>
    `}static get styles(){let t=e`var(--circularActivityIndicatorColor, dodgerblue)`,n=e`var(--circularActivityIndicatorThickness, 4px)`;return e`
      :host {
        display: block;
      }

      .lds-ring {
        display: inline-block;
        position: relative;
        width: 100%;
        height: 100%;
      }
      .lds-ring div {
        box-sizing: border-box;
        display: block;
        position: absolute;
        width: 80%;
        height: 80%;
        margin: ${n};
        border-width: ${n};
        border-style: solid;
        border-radius: 50%;
        animation: lds-ring 1.2s cubic-bezier(0.5, 0, 0.5, 1) infinite;
        border-color: ${t} transparent transparent transparent;
      }
      .lds-ring div:nth-child(1) {
        animation-delay: -0.45s;
      }
      .lds-ring div:nth-child(2) {
        animation-delay: -0.3s;
      }
      .lds-ring div:nth-child(3) {
        animation-delay: -0.15s;
      }
      @keyframes lds-ring {
        0% {
          transform: rotate(0deg);
        }
        100% {
          transform: rotate(360deg);
        }
      }
    `}};Gi=C([x(`circular-activity-indicator`)],Gi);function Ki(e,t){return e===void 0?t!==void 0:t===void 0?e!==void 0:e.facetType===t.facetType&&e.bucketKey===t.bucketKey}function qi(e,t){return e.label===t.label&&e.facets.length===t.facets.length&&e.facets.every((e,n)=>Ki(e,t.facets[n]))}function Ji(e){if(!Array.isArray(e[0])){let t=[...e];for(let e=0;e<t.length;e++){let n=t[e];t=t.filter(e=>n===e||!qi(n,e))}return t}let t=e,n=[];for(let e of t){let t=[];for(let r of e)n.find(e=>e.find(e=>qi(r,e)))||t.push(r);t.length>0&&n.push(t)}return n}var Yi=class e{async getRecommendedFacets(){let t=navigator.language,n=e.getLanguageDisplayName(t);return n?[{facets:[{facetType:`language`,bucketKey:n}]}]:[]}static getLanguageDisplayName(e){let t=e.split(`-`)[0];return new Intl.DisplayNames([`en`],{type:`language`}).of(t)}},Xi={facets:[{facetType:`mediatype`,bucketKey:`texts`}]},Zi={facets:[{facetType:`mediatype`,bucketKey:`audio`}]},Qi={facets:[{facetType:`mediatype`,bucketKey:`movies`}]},$i={facets:[{facetType:`mediatype`,bucketKey:`image`}]},ea={facets:[{facetType:`mediatype`,bucketKey:`software`}]},ta={facets:[{facetType:`mediatype`,bucketKey:`etree`}]},na={text:[Xi],book:[Xi],novel:[Xi],magazine:[Xi],newspaper:[Xi],pdf:[Xi],epub:[Xi],audio:[Zi],song:[Zi],music:[Zi],listen:[Zi],podcast:[Zi],radio:[Zi],stream:[Zi,Qi],video:[Qi],movie:[Qi],film:[Qi],animation:[Qi],youtube:[Qi],image:[$i],photo:[$i],picture:[$i],painting:[$i],jpg:[$i],jpeg:[$i],png:[$i],gif:[$i],software:[ea],app:[ea],program:[ea],game:[ea],arcade:[ea],etree:[ta],concert:[ta],"live music":[ta],"web crawl":[{facets:[{facetType:`mediatype`,bucketKey:`web`}]}],dataset:[{facets:[{facetType:`mediatype`,bucketKey:`data`}]}]},ra=class{async getRecommendedFacets(e){let t=[];for(let[n,r]of Object.entries(na))e.includes(n)&&t.push(...r);return t}},ia=[{facets:[{facetType:`mediatype`,bucketKey:`texts`}]}],aa=[{get label(){return o(`Films by __QUERY`)},facets:[{facetType:`mediatype`,bucketKey:`movies`},{facetType:`creator`,bucketKey:`__QUERY`}]}],oa=[{get label(){return o(`Writing by __QUERY`)},facets:[{facetType:`mediatype`,bucketKey:`texts`},{facetType:`creator`,bucketKey:`__QUERY`}]}],sa=[{get label(){return o(`Images by __QUERY`)},facets:[{facetType:`mediatype`,bucketKey:`image`},{facetType:`creator`,bucketKey:`__QUERY`}]}],ca=[{get label(){return o(`Music by __QUERY`)},facets:[{facetType:`mediatype`,bucketKey:`audio`},{facetType:`creator`,bucketKey:`__QUERY`}]}],la={"written work":ia,literature:ia,book:ia,novel:ia,filmmaker:aa,director:aa,author:oa,writer:oa,novelist:oa,essayist:oa,poet:oa,"visual artist":sa,"graphic artist":sa,photographer:sa,painter:sa,singer:ca,songwriter:ca,musician:ca,composer:ca,pianist:ca},ua=class{constructor(){this.WIKIDATA_BASE_URL=`https://www.wikidata.org/w/api.php`,this.WIKIDATA_DEFAULT_ARGS=`?action=wbsearchentities&format=json&language=en&uselang=en&origin=*&type=item&limit=5`}getWikidataURL(e){let t=encodeURIComponent(e);return`${this.WIKIDATA_BASE_URL}${this.WIKIDATA_DEFAULT_ARGS}&search=${t}`}replaceQueryPlaceholders(e,t){return e.map(e=>({label:e.label?.replace(`__QUERY`,t),facets:e.facets.map(e=>{let n={...e,bucketKey:e.bucketKey.replace(`__QUERY`,t.toLowerCase())};return e.displayText&&(n.displayText=n.displayText?.replace(`__QUERY`,t)),n})}))}async getRecommendedFacets(e){let t=[];try{let n=this.getWikidataURL(e),r=await(await fetch(n)).json();for(let[e,n]of Object.entries(la))if(RegExp(`\\b${e}\\b`).test(r.search[0]?.description)){let e=r.search[0].label;t.push(...this.replaceQueryPlaceholders(n,e))}return t}catch(e){return console.warn(e),[]}}},da,fa=class e{async getRecommendedFacets(t,n=e.DEFAULT_HEURISTICS){let r=n.map(e=>new e().getRecommendedFacets(t));return Ji((await Promise.all(r)).flat())}};da=fa,da.DEFAULT_HEURISTICS=[ra,ua,Yi];var pa=a`
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <path
      d="m 91.666668,8.3333328 v 0.9708343 l -35.3625,39.2916669 -2.137502,2.375 v 3.195832 32.350001 L 45.833334,82.35 V 54.166666 50.970834 l -2.1375,-2.375 L 8.3333328,9.3041671 V 8.3333328 H 91.666668 M 100,0 H 0 V 12.5 L 37.500001,54.166666 V 87.5 l 25,12.5 V 54.166666 L 100,12.5 Z"
      fill="#000"
    />
  </svg>
`;function ma(e){return e&&e.charAt(0).toLocaleUpperCase()+e.slice(1)}var ha=class extends c{constructor(){super(...arguments),this.selected=!1}render(){if(!this.facetInfo)return n;let e=this.facetInfo.facets.length===1,t=this.facetInfo.facets[0],r=ma((this.labelPrefix?`${this.labelPrefix} `:``)+(this.facetInfo.label??t.displayText??t.bucketKey));if(!r)return n;let i=e&&t.facetType===`mediatype`?Ht[t.bucketKey].icon:n;return a`
      <a
        class="smart-facet-button ${this.selected?`selected`:``}"
        href=${this.href}
        @click=${this.facetClicked}
      >
        ${i} ${r}
        ${this.selected?a`<span class="unselect-button">${Ii}</span>`:n}
      </a>
    `}get href(){let e=new URL(window.location.href);if(this.facetInfo)for(let t of this.facetInfo.facets)e.searchParams.append(`and[]`,encodeURIComponent(`${t.facetType}:"${t.bucketKey}"`));return e.toString()}facetClicked(e){e.preventDefault(),this.facetInfo&&(this.selected=!this.selected,this.dispatchEvent(new CustomEvent(`facetClick`,{detail:{smartFacet:this.facetInfo,details:this.facetInfo.facets.map(e=>({facetType:e.facetType,bucket:{key:e.bucketKey,count:0,state:this.selected?`selected`:`none`},negative:!1}))}})))}static get styles(){return e`
      .smart-facet-button {
        display: inline-flex;
        align-items: center;
        column-gap: 5px;
        padding: 5px 5px;
        border-radius: 5px;
        background: white;
        color: #2c2c2c;
        border: 1px solid #194880;
        font-size: 1.4rem;
        font-family: inherit;
        line-height: normal;
        text-decoration: none;
      }

      .smart-facet-button.selected {
        background: #194880;
        color: white;
      }

      .unselect-button > svg {
        width: 10px;
        height: 10px;
        filter: invert(1);
      }

      .smart-facet-button > svg {
        width: 12px;
        height: 12px;
        filter: invert(0.16667);
      }

      .smart-facet-button.selected > svg {
        filter: invert(1);
      }
    `}};C([t({type:Object})],ha.prototype,`facetInfo`,void 0),C([t({type:String})],ha.prototype,`labelPrefix`,void 0),C([t({type:Boolean})],ha.prototype,`selected`,void 0),ha=C([x(`smart-facet-button`)],ha);var ga=class extends c{render(){if(!this.facetInfo||!this.activeFacetRef||this.facetInfo.length===0)return n;let e=this.activeFacetRef.displayText??this.activeFacetRef.bucketKey;return e?a`
      <div class="dropdown-container">
        <ia-dropdown
          class="dropdown"
          displayCaret
          closeOnSelect
          closeOnEscape
          closeOnBackdropClick
          includeSelectedOption
          usePopover
          .options=${this.dropdownOptions}
          .selectedOption=${this.activeDropdownOption}
          .openViaButton=${!1}
          @optionSelected=${this.optionSelected}
          @click=${this.onDropdownClick}
        >
          <span
            class="dropdown-label"
            slot="dropdown-label"
            @click=${this.defaultOptionSelected}
            >${this.labelPrefix??n} ${e}</span
          >
        </ia-dropdown>
      </div>
    `:n}get dropdownOptions(){return this.facetInfo?.map(e=>{let t=e.facets[0];return{id:t.bucketKey,label:a`<span>
            ${e.label??t.displayText??t.bucketKey}
          </span>`}})??[]}get activeDropdownOption(){if(this.activeFacetRef)return this.dropdownOptions.find(e=>e.id===this.activeFacetRef?.bucketKey)}defaultOptionSelected(){this.handleSelection(this.activeFacetRef?.bucketKey)}optionSelected(e){this.handleSelection(e.detail.option.id)}handleSelection(e){if(!e||!this.facetInfo||!this.activeFacetRef)return;let t;for(let n of this.facetInfo){let r=n.facets.find(t=>t.bucketKey===e);r&&(this.activeFacetRef=r,t=n)}t&&this.dispatchEvent(new CustomEvent(`facetClick`,{detail:{smartFacet:t,details:[{facetType:this.activeFacetRef.facetType,bucket:{key:this.activeFacetRef.bucketKey,count:0,state:`selected`},negative:!1}]}}))}onDropdownClick(){Y(`smart dropdown: onDropdownClick`,this),this.dispatchEvent(new CustomEvent(`dropdownClick`,{detail:this}))}close(){this.dropdown&&(this.dropdown.open=!1)}static get styles(){return e`
      .dropdown-container {
        padding: 5px 5px;
        border-radius: 5px;
        background: white;
        color: #2c2c2c;
        border: 1px solid #194880;
        font-size: 1.4rem;
        font-family: inherit;
      }

      .dropdown-label {
        font-size: 1.4rem;
        font-family: inherit;
      }

      .dropdown {
        --dropdownBorderColor: #194880;
        --dropdownBorderWidth: 1px;
        --dropdownBgColor: white;
        --dropdownHoverBgColor: #f8f8f8;
        --dropdownTextColor: #2c2c2c;
        --dropdownHoverTextColor: #2c2c2c;
        --dropdownCaretColor: #2c2c2c;
        --dropdownWhiteSpace: nowrap;
        --caretWidth: 14px;
        --caretHeight: 14px;
      }
    `}};C([t({type:Array})],ga.prototype,`facetInfo`,void 0),C([t({type:String})],ga.prototype,`labelPrefix`,void 0),C([t({type:Object})],ga.prototype,`activeFacetRef`,void 0),C([S(`ia-dropdown`)],ga.prototype,`dropdown`,void 0),ga=C([x(`smart-facet-dropdown`)],ga);var _a={collection:`Collection: `,creator:`By: `,subject:`About: `};function va(e){return e.charAt(0).toUpperCase()+e.slice(1)}var Q=class extends c{constructor(){super(...arguments),this.filterToggleShown=!1,this.filterToggleActive=!1,this.heuristicRecs=[],this.smartFacets=[]}render(){if(!this.query)return n;let e=!!this.label&&this.smartFacets.length>0;return a`
      <div id="smart-facets-container">
        ${this.filtersToggleTemplate}
        ${e?a`<p id="filters-label">${this.label}</p>`:n}
        ${v(this.smartFacets,e=>`${e[0].label}|${e[0].facets[0].facetType}|${e[0].facets[0].bucketKey}`,e=>this.makeSmartFacet(e))}
      </div>
    `}willUpdate(e){let t=!1;e.has(`query`)&&(Y(`query change`,e.get(`query`),this.query),this.lastAggregations=void 0,t=!0),e.has(`aggregations`)&&!this.lastAggregations&&this.aggregations&&Object.keys(this.aggregations).length>0&&(Y(`aggs change`,e.get(`aggregations`),this.aggregations),this.lastAggregations=this.aggregations,t=!0),t&&(Y(`should update smart facets, doing so...`),this.updateSmartFacets())}refresh(){this.lastAggregations=this.aggregations,this.updateSmartFacets()}deselectAll(){for(let e of this.smartFacets)for(let t of e)t.selected=!1;this.requestUpdate()}async updateSmartFacets(){Y(`updating smart facets`),this.query&&(this.heuristicRecs=await new fa().getRecommendedFacets(this.query),Y(`heuristic recs are`,this.heuristicRecs),this.smartFacets=Ji(this.facetsToDisplay),Y(`smart facets are`,this.smartFacets))}makeSmartFacet(e){return e.length===0?n:e.length===1?this.smartFacetButton(e[0]):this.smartFacetDropdown(e)}smartFacetButton(e){return a`
      <smart-facet-button
        .facetInfo=${e}
        .labelPrefix=${_a[e.facets[0].facetType]}
        .selected=${e.selected??!1}
        @facetClick=${this.facetClicked}
      ></smart-facet-button>
    `}smartFacetDropdown(e){return a`
      <smart-facet-dropdown
        .facetInfo=${e}
        .labelPrefix=${_a[e[0].facets[0].facetType]}
        .activeFacetRef=${e[0].facets[0]}
        @facetClick=${this.dropdownOptionClicked}
        @dropdownClick=${this.dropdownClicked}
      ></smart-facet-dropdown>
    `}get filtersToggleTemplate(){return this.filterToggleShown?a`
      <button
        id="filters-toggle"
        class=${this.filterToggleActive?`active`:``}
        title=${this.filterToggleActive?o(`Hide filters pane`):o(`Show filters pane`)}
        @click=${this.filterToggleClicked}
      >
        ${pa}
      </button>
    `:n}get facetsToDisplay(){let e=[];if(this.heuristicRecs.length>0)for(let t of this.heuristicRecs)t.facets.length===1&&t.facets[0].facetType===`mediatype`||e.push([t]);if(this.lastAggregations)for(let t of[`mediatype`,`year`,`language`,`creator`,`subject`,`collection`]){let n=this.lastAggregations[t];if(!n||n.buckets.length===0||[`lending`,`year_histogram`,`date_histogram`].includes(t)||typeof n.buckets[0]==`number`||t===`mediatype`&&this.selectedFacets&&Object.values(this.selectedFacets.mediatype??{}).some(e=>e.state!==`none`))continue;let r=t,i=n.buckets.filter(e=>{let t=this.selectedFacets?.[r]?.[e.key];return!(t&&t.state!==`none`)});if(r!==`mediatype`)if(r===`collection`||r===`subject`){let t=i.slice(0,5);e.push(t.map(e=>this.toSmartFacet(r,[e])))}else e.push([this.toSmartFacet(r,[i[0]])])}return e}toSmartFacet(e,t){return{facets:t.map(t=>{let n=va(t.key.toString());if(e===`collection`){let e=this.collectionTitles?.get(t.key.toString());e&&(n=e)}return{facetType:e,bucketKey:t.key.toString(),displayText:n}})}}toggleSmartFacet(e,t){let n;e.selected?(n=`none`,this.smartFacets=this.smartFacets.map(t=>t[0]===e?[{...e,selected:!1}]:t)):(n=`selected`,this.smartFacets=[[{...e,selected:!0}],...this.smartFacets.filter(t=>t[0]!==e)]),this.updateSelectedFacets(t.map(e=>({...e,bucket:{...e.bucket,state:n}})))}updateSelectedFacets(e){for(let t of e)this.selectedFacets=He(this.selectedFacets,t.facetType,t.bucket,!0);let t=new CustomEvent(`facetsChanged`,{detail:this.selectedFacets});this.dispatchEvent(t)}facetClicked(e){this.toggleSmartFacet(e.detail.smartFacet,e.detail.details)}dropdownOptionClicked(e){let t=this.smartFacets.find(t=>t.length===1&&qi(t[0],e.detail.smartFacet));if(t){this.toggleSmartFacet(t[0],e.detail.details);return}this.smartFacets=[[{...e.detail.smartFacet,selected:!0}],...this.smartFacets],this.updateSelectedFacets(e.detail.details)}dropdownClicked(e){Y(`smart bar: onDropdownClick`,e.detail),this.shadowRoot?.querySelectorAll(`smart-facet-dropdown`).forEach(t=>{t!==e.detail&&(Y(`closing`,t),t.close())})}filterToggleClicked(){this.dispatchEvent(new CustomEvent(`filtersToggled`))}static get styles(){return e`
      #smart-facets-container {
        display: flex;
        align-items: center;
        gap: 5px 10px;
        padding: 10px 0;
        white-space: nowrap;
        overflow: scroll hidden;
        scrollbar-width: none;
      }

      #filters-toggle {
        margin: 0;
        border: 0;
        padding: 5px 8px;
        border-radius: 5px;
        background: white;
        color: #2c2c2c;
        border: 1px solid #194880;
        font-size: 1.4rem;
        font-family: inherit;
        text-decoration: none;
        cursor: pointer;
      }

      #filters-toggle.active {
        background: #194880;
        color: white;
      }

      #filters-toggle > svg {
        width: 12px;
        filter: invert(0.16667);
        vertical-align: -1px;
      }

      #filters-toggle.active > svg {
        filter: invert(1);
      }

      #filters-label {
        font-size: 1.4rem;
        font-weight: var(--smartFacetLabelFontWeight, normal);
        margin: 0 -5px 0 0;
      }
    `}};C([t({type:String})],Q.prototype,`query`,void 0),C([t({type:Object})],Q.prototype,`aggregations`,void 0),C([t({type:Object})],Q.prototype,`selectedFacets`,void 0),C([t({type:Object})],Q.prototype,`collectionTitles`,void 0),C([t({type:Boolean})],Q.prototype,`filterToggleShown`,void 0),C([t({type:Boolean})],Q.prototype,`filterToggleActive`,void 0),C([t({type:String})],Q.prototype,`label`,void 0),C([s()],Q.prototype,`heuristicRecs`,void 0),C([s()],Q.prototype,`smartFacets`,void 0),C([s()],Q.prototype,`lastAggregations`,void 0),Q=C([x(`smart-facet-bar`),i()],Q);var ya,ba,$=(ya=class extends c{constructor(){super(),this.baseImageUrl=`https://archive.org`,this.searchType=w.DEFAULT,this.selectedSort=E.default,this.selectedTitleFilter=null,this.selectedCreatorFilter=null,this.sortDirection=null,this.defaultSortField=E.relevance,this.defaultSortDirection=null,this.pageSize=50,this.showSmartFacetBar=!1,this.showHistogramDatePicker=!1,this.suppressPlaceholders=!1,this.suppressResultCount=!1,this.suppressResultTiles=!1,this.suppressURLQuery=!1,this.suppressURLSinParam=!1,this.suppressSortBar=!1,this.suppressDisplayModes=!1,this.suppressMediatypeFacets=!1,this.facetLoadStrategy=`eager`,this.facetPaneVisible=!1,this.clearResultsOnEmptyQuery=!1,this.collectionPagePath=`/details/`,this.searchContext=k.default,this.pageContext=`search`,this.restorationStateHandler=new Le({context:this.pageContext}),this.mobileBreakpoint=600,this.loggedIn=!1,this.modalManager=void 0,this.isTVCollection=!1,this.isManageView=!1,this.tileActions=[],this.tileLayoutType=`default`,this.enableSortOptionsSlot=!1,this.showSmartResults=!1,this.maxPagesToManage=15,this.dataSource=new qe(this,this.pageSize),this.initialPageNumber=1,this.pagesToRender=this.initialPageNumber,this.searchResultsLoading=!1,this.facetsLoading=!1,this.mobileView=!1,this.tileBlurOverrideState=`no-override`,this.collapsibleFacetsVisible=!1,this.placeholderType=null,this.selectedTVNetwork=void 0,this.selectedTVShow=void 0,this.tvMapsPopulated=!1,this.tvMapsLoading=!1,this.tvMapsErrored=!1,this.layoutSizeAnalyticsSent=!1,this.isScrollingToCell=!1,this.isResizeToMobile=!1,this.dataSourceInstallInProgress=!1,this.placeholderCellTemplate=a`<collection-browser-loading-tile></collection-browser-loading-tile>`,this.deferredFetchTimer=0,this.updateLeftColumnHeight=()=>{if(this.mobileView)this.leftColumn?.style?.removeProperty(`height`);else{let e=this.leftColumn?.getBoundingClientRect().top,t=window.innerHeight-(e??0),n=e===0?null:`calc(${t}px - var(--leftColumnPaddingTop, 2rem))`;this.leftColumn?.style?.setProperty(`height`,n)}},this.updateFacetFadeOut=e=>{(this.shadowRoot?.getElementById(`facets-bottom-fade`))?.classList.toggle(`hidden`,e?.[0]?.isIntersecting)},this.initialQueryChangeHappened=!1,this.historyPopOccurred=!1,this.addController(this.dataSource)}tileModelAtCellIndex(e){let t=this.dataSource.getTileModelAt(e);if(!t&&!this.isScrollingToCell&&this.dataSource.queryInitialized){let t=Math.floor(e/this.pageSize)+1;this.scheduleDeferredPageFetch(t)}return t}scheduleDeferredPageFetch(e){this.deferredFetchTimer?window.clearTimeout(this.deferredFetchTimer):this.dataSource.fetchPage(e),this.deferredFetchTimer=window.setTimeout(()=>{this.deferredFetchTimer=0,this.fetchVisiblePages()},ba.DEFERRED_FETCH_DELAY_MS)}fetchVisiblePages(){let e=this.infiniteScroller?.getVisibleCellIndices()??[],t=new Set(e.map(e=>Math.floor(e/this.pageSize)+1));for(let e of t){let t=(e-1)*this.pageSize;this.dataSource.getTileModelAt(t)||this.dataSource.fetchPage(e)}}get estimatedTileCount(){return this.pagesToRender*this.pageSize}async getSessionId(){try{let e=sessionStorage?.getItem(`cb-session`);if(e)return e;if(this.sessionIdGenPromise)return this.sessionIdGenPromise;this.sessionIdGenPromise=Be(Math.random().toString());let t=await this.sessionIdGenPromise;return sessionStorage?.setItem(`cb-session`,t),t}catch{return``}}goToPage(e){return this.initialPageNumber=e,this.pagesToRender=e,this.scrollToPage(e)}setSearchResultsLoading(e){this.searchResultsLoading=e}setFacetsLoading(e){this.facetsLoading=e}setTotalResultCount(e){this.totalResults=e}clearFilters({facets:e=!0,dateRange:t=!0,letterFilters:n=!0,sort:r=!1}={}){e&&this.hasCheckedFacets&&(this.selectedFacets=O()),t&&(this.minSelectedDate=void 0,this.maxSelectedDate=void 0),n&&(this.selectedTitleFilter=null,this.selectedCreatorFilter=null),r&&(this.sortDirection=null,this.selectedSort=E.default),this.clearTVDropdowns(),this.smartFacetBar&&this.smartFacetBar.deselectAll()}clearTVDropdowns(){this.selectedTVNetwork=void 0,this.selectedTVShow=void 0,this.tvNetworksDropdown&&this.tvNetworksDropdown.clearSelectedOption(),this.tvShowsDropdown&&this.tvShowsDropdown.clearSelectedOption()}get hasCheckedFacets(){if(!this.selectedFacets)return!1;for(let e of Object.values(this.selectedFacets))for(let t of Object.values(e))if(t.state!==`none`)return!0;return!1}get hasActiveFilters(){return!!(this.hasCheckedFacets||this.minSelectedDate||this.maxSelectedDate||this.selectedTitleFilter||this.selectedCreatorFilter)}willUpdate(e){if(this.setPlaceholderType(),e.has(`searchType`)&&this.searchType===w.TV&&this.applyDefaultTVSearchSort(),(e.has(`baseQuery`)||e.has(`identifiers`)||e.has(`searchType`)||e.has(`withinCollection`))&&!this.historyPopOccurred&&this.initialQueryChangeHappened){let t=e.has(`withinCollection`)&&!e.has(`selectedSort`)&&!e.has(`sortDirection`);this.clearFilters({sort:t,facets:!e.has(`selectedFacets`),dateRange:!(e.has(`minSelectedDate`)||e.has(`maxSelectedDate`)),letterFilters:!(e.has(`selectedTitleFilter`)||e.has(`selectedCreatorFilter`))})}}render(){return a`
      ${this.showSmartFacetBar&&this.placeholderType===null?a`<smart-facet-bar
            .query=${this.baseQuery}
            .aggregations=${this.dataSource.aggregations}
            .selectedFacets=${this.selectedFacets}
            .collectionTitles=${this.dataSource.collectionTitles}
            .filterToggleShown=${!this.mobileView}
            .filterToggleActive=${this.facetPaneVisible}
            .label=${this.smartFacetBarLabel}
            @facetsChanged=${this.facetsChanged}
            @filtersToggled=${()=>{this.facetPaneVisible=!this.facetPaneVisible,this.emitFacetPaneVisibilityChanged()}}
          ></smart-facet-bar>`:n}

      <div
        id="content-container"
        class=${this.mobileView?`mobile`:`desktop`}
      >
        ${this.placeholderType?this.emptyPlaceholderTemplate:this.collectionBrowserTemplate}
      </div>
    `}setPlaceholderType(){let e=this.dataSource.queryInitialized,t=!!this.baseQuery?.trim(),n=!!this.identifiers?.length,r=!!this.withinCollection,i=!!this.withinProfile,a=!r&&!i,o=!this.searchResultsLoading&&(this.dataSource.size===0||!this.searchService);this.placeholderType=null,!this.suppressPlaceholders&&(e?a&&!t&&!n?this.placeholderType=`empty-query`:o&&(this.placeholderType=!t&&r?`empty-collection`:`no-results`):this.placeholderType=`empty-query`,this.dataSource.queryErrorMessage&&(this.placeholderType=!t&&r?`collection-error`:`query-error`))}get emptyPlaceholderTemplate(){return a`
      <empty-placeholder
        .placeholderType=${this.placeholderType}
        ?isMobileView=${this.mobileView}
        ?isCollection=${!!this.withinCollection}
        .detailMessage=${this.dataSource.queryErrorMessage??``}
        .baseNavigationUrl=${this.baseNavigationUrl}
      ></empty-placeholder>
    `}get collectionBrowserTemplate(){return a`
      <div id="left-column-scroll-sentinel"></div>
      ${this.leftColumnTemplate} ${this.rightColumnTemplate}
    `}get leftColumnTemplate(){return this.mobileView?this.mobileLeftColumnTemplate:this.desktopLeftColumnTemplate}get mobileLeftColumnTemplate(){return a`
      <div
        id="left-column"
        class="column${this.isResizeToMobile?` preload`:``}"
      >
        ${this.facetTopViewSlot} ${this.resultsCountTemplate}
        <div id="facets-header-container">${this.mobileFacetsTemplate}</div>
      </div>
    `}get desktopLeftColumnTemplate(){return a`
      <div id="left-column" class="column" ?hidden=${!this.facetPaneVisible}>
        ${this.facetTopViewSlot}
        <div id="facets-header-container">
          <h2 id="facets-header" class="sr-only">${o(`Filters`)}</h2>
          ${this.resultsCountTemplate} ${this.clearFiltersBtnTemplate(!1)}
        </div>
        <div id="facets-container" aria-labelledby="facets-header">
          ${this.facetsTemplate}
          <div id="facets-scroll-sentinel"></div>
        </div>
        <div id="facets-bottom-fade"></div>
      </div>
    `}get facetTopViewSlot(){return a`<div id="facet-top-view">
      <slot name="facet-top-slot"></slot>
    </div>`}get resultsCountTemplate(){if(this.suppressResultCount)return n;let e=this.searchResultsLoading||this.totalResults===void 0,t=g({filtered:this.hasActiveFilters}),r=this.totalResults===void 0?void 0:new Intl.NumberFormat(M()).format(this.totalResults),i=this.totalResults===1?o(`Result`):o(`Results`);return a`
      <div id="results-total" class=${t} data-testid="results-total">
        <span id="big-results-count">
          ${e?o(`Searching…`):r}
        </span>
        <span id="big-results-label">
          ${e?n:i}
        </span>
      </div>
    `}get rightColumnTemplate(){return a`
      <div id="right-column" class=${g({column:!0,"full-width":!this.facetPaneVisible,"smart-results-spacing":!!this.showSmartResults})}>
        ${this.showSmartResults?a`<slot name="smart-results"></slot>`:n}
        <section id="results">
          <h2 class="results-section-heading">
            <slot name="results-heading"></slot>
          </h2>
          <div id="cb-top-view">
            <slot name="cb-top-slot"></slot>
          </div>
          ${this.isManageView?this.manageBarTemplate:this.sortFilterBarTemplate}
          <slot name="cb-results"></slot>
          <div id="results-body">
            <slot name="results-aside"></slot>
            <div id="results-main">
              ${this.displayMode===`list-compact`&&this.totalResults?this.listHeaderTemplate:n}
              ${this.suppressResultTiles?n:this.infiniteScrollerTemplate}
            </div>
          </div>
        </section>
      </div>
    `}get infiniteScrollerTemplate(){return a`<infinite-scroller
      class=${this.infiniteScrollerClasses}
      itemCount=${this.placeholderType?0:n}
      ariaLandmarkLabel=${o(`Search results`)}
      .estimatedCellHeight=${this.estimatedTileHeight}
      .minBufferMarginCells=${this.pageSize}
      .cellProvider=${this}
      .placeholderCellTemplate=${this.placeholderCellTemplate}
      @scrollThresholdReached=${this.scrollThresholdReached}
      @visibleCellsChanged=${this.visibleCellsChanged}
      >${this.displayMode===`grid`?a`<slot name="result-last-tile" slot="result-last-tile"></slot>`:n}
    </infinite-scroller>`}get infiniteScrollerClasses(){return g({[this.displayMode??``]:!!this.displayMode,hidden:!!this.placeholderType})}get estimatedTileHeight(){switch(this.displayMode){case`list-detail`:return 80;case`list-compact`:return 45;default:return 225}}get sortFilterBarTemplate(){if(this.suppressSortBar)return n;let e=ve;return this.withinCollection?.startsWith(`fav-`)||this.profileElement===`favorites`?e=ye:!this.withinCollection&&this.searchType===w.TV&&(e=be),e.relevance=this.isRelevanceSortAvailable,a`
      <sort-filter-bar
        .defaultSortField=${this.defaultSortField}
        .defaultSortDirection=${this.defaultSortDirection}
        .selectedSort=${this.selectedSort}
        .sortDirection=${this.sortDirection}
        .sortFieldAvailability=${e}
        .displayMode=${this.displayMode}
        .selectedTitleFilter=${this.selectedTitleFilter}
        .selectedCreatorFilter=${this.selectedCreatorFilter}
        .prefixFilterCountMap=${this.dataSource.prefixFilterCountMap}
        .enableSortOptionsSlot=${this.enableSortOptionsSlot}
        .suppressDisplayModes=${this.suppressDisplayModes}
        @sortChanged=${this.userChangedSort}
        @displayModeChanged=${this.displayModeChanged}
        @titleLetterChanged=${this.titleLetterSelected}
        @creatorLetterChanged=${this.creatorLetterSelected}
      >
        ${this.tileBlurCheckboxTemplate}
        <slot name="sort-options-left" slot="sort-options-left"></slot>
        <slot name="sort-options" slot="sort-options"></slot>
        <slot name="sort-options-right" slot="sort-options-right"></slot>
        <slot name="sort-bar-end" slot="sort-bar-end"></slot>
      </sort-filter-bar>
    `}get tileBlurCheckboxTemplate(){return this.dataSource.sessionContext?.is_archive_user?a`
      <label
        id="tile-blur-label"
        for="tile-blur-check"
        slot="sort-options-right"
      >
        ${o(`Blurring`)}
        <input
          id="tile-blur-check"
          type="checkbox"
          ?checked=${!this.shouldSuppressTileBlurring}
          @change=${this.tileBlurCheckboxChanged}
        />
      </label>
    `:n}get manageBarTemplate(){return a`
      <manage-bar
        .label=${this.manageViewLabel}
        .modalManager=${this.modalManager}
        .selectedItems=${this.dataSource.checkedTileModels}
        .profileElement=${this.profileElement}
        showSelectAll
        showUnselectAll
        ?showItemManageButton=${this.pageContext===`search`}
        ?removeAllowed=${this.dataSource.checkedTileModels.length!==0}
        @removeItems=${this.handleRemoveItems}
        @manageItems=${this.handleManageItems}
        @selectAll=${()=>this.dataSource.checkAllTiles()}
        @unselectAll=${()=>this.dataSource.uncheckAllTiles()}
        @cancel=${()=>{this.isManageView=!1,this.dataSource.uncheckAllTiles(),this.searchResultsLoading&&this.dataSource.resetPages()}}
      ></manage-bar>
    `}handleRemoveItems(){this.dispatchEvent(new CustomEvent(`itemRemovalRequested`,{detail:{items:this.dataSource.checkedTileModels.map(e=>`${e?.mediatype===`search`?`search:`:``}${e?.identifier??``}`)}}))}handleManageItems(){this.dispatchEvent(new CustomEvent(`itemManagerRequested`,{detail:{items:this.dataSource.checkedTileModels.map(e=>e?.identifier?e.identifier:``)}}))}refreshSmartFacets(){this.smartFacetBar?.refresh()}showRemoveItemsProcessingModal(){this.manageBar?.showRemoveItemsProcessingModal()}showRemoveItemsErrorModal(){this.manageBar?.showRemoveItemsErrorModal()}removeCheckedTiles(){this.dataSource.removeCheckedTiles()}tileBlurCheckboxChanged(e){let{checked:t}=e.target;this.tileBlurOverrideState=t?`on`:`off`,this.infiniteScroller?.refreshAllVisibleCells()}get shouldSuppressTileBlurring(){if(this.tileBlurOverrideState!==`no-override`)return this.tileBlurOverrideState===`off`;let{sessionContext:e}=this.dataSource;return e?.pps_relevant_user_preferences?.display__blur_moderated_content===`off`}userChangedSort(e){let{selectedSort:t,sortDirection:n}=e.detail;this.selectedSort=t,this.sortDirection=n,(this.currentPage??1)>1&&this.goToPage(1),this.setCurrentPage(1)}sendSortByAnalytics(e){let t=e&&!this.sortDirection;this.analyticsHandler?.sendEvent({category:this.searchContext,action:A.sortBy,label:`${this.selectedSort}${this.sortDirection||t?`-${this.sortDirection}`:``}`})}selectedSortChanged(){this.dataSource.updatePrefixFiltersForCurrentSort()}get sortParam(){let e=D[this.selectedSort];if(!e?.handledBySearchService)return null;let t=new URL(window.location.href).searchParams.get(`sort`),n=e.searchServiceKey??t?.replace(/^-/,``);return this.sortDirection||(this.sortDirection=`asc`),n?{field:n,direction:this.sortDirection}:null}get defaultSortParam(){let e=this.defaultSortDirection??`asc`,t=D[this.defaultSortField].searchServiceKey;return t?{field:t,direction:e}:null}displayModeChanged(e){this.displayMode=e.detail.displayMode,this.displayMode&&this.analyticsHandler?.sendEvent({category:this.searchContext,action:A.displayMode,label:this.displayMode})}get titleQuery(){return this.selectedTitleFilter?`firstTitle:${this.selectedTitleFilter}`:void 0}get creatorQuery(){return this.selectedCreatorFilter?`firstCreator:${this.selectedCreatorFilter}`:void 0}sendFilterByTitleAnalytics(e){if(!e&&!this.selectedTitleFilter)return;let t=e&&!this.selectedTitleFilter;this.analyticsHandler?.sendEvent({category:this.searchContext,action:A.filterByTitle,label:t?`clear-${e}`:`${e||`start`}-${this.selectedTitleFilter}`})}sendFilterByCreatorAnalytics(e){if(!e&&!this.selectedCreatorFilter)return;let t=e&&!this.selectedCreatorFilter;this.analyticsHandler?.sendEvent({category:this.searchContext,action:A.filterByCreator,label:t?`clear-${e}`:`${e||`start`}-${this.selectedCreatorFilter}`})}titleLetterSelected(e){this.selectedCreatorFilter=null,this.selectedTitleFilter=e.detail.selectedLetter}creatorLetterSelected(e){this.selectedTitleFilter=null,this.selectedCreatorFilter=e.detail.selectedLetter}get mobileFacetsTemplate(){return Re.includes(this.profileElement)?n:a`
      <details id="mobile-filter-collapse" @toggle=${e=>{let t=e.target;this.isResizeToMobile=!1,this.collapsibleFacetsVisible=t.open,this.analyticsHandler?.sendEvent({category:this.searchContext,action:A.mobileFacetsToggled,label:t.open?`open`:`closed`})}}>
        <summary>
          <span class="collapser-icon">${Je}</span>
          <h2>${o(`Filters`)}</h2>
          ${this.clearFiltersBtnTemplate(!0)}
        </summary>
        ${this.facetsTemplate}
      </details>
    `}async tvDropdownToggled(e){if(e.detail&&!this.tvMapsPopulated){this.tvMapsLoading=!0,this.tvMapsErrored=!1;try{await this.dataSource.populateTVChannelMaps(),this.tvMapsPopulated=!0}catch{this.tvMapsErrored=!0}this.tvMapsLoading=!1}}async networksDropdownChanged(){let e=this.selectedTVNetwork,t=this.tvNetworksDropdown.selectedOption?.text;this.selectedTVNetwork=t??void 0;let n=this.dataSource.tvChannelMaps.channelToNetwork.entries();for(let[t,r]of n)if(r===e){let e={key:t.toLowerCase(),count:0,state:`none`};this.selectedFacets=He(this.selectedFacets,`creator`,e,!0)}else if(r===this.selectedTVNetwork){let e={key:t.toLowerCase(),count:0,state:`selected`};this.selectedFacets=He(this.selectedFacets,`creator`,e)}}async showsDropdownChanged(){let e=this.selectedTVShow,t=this.tvShowsDropdown.selectedOption?.text;if(this.selectedTVShow=t??void 0,e!==void 0){let t={key:e,count:0,state:`none`};this.selectedFacets=He(this.selectedFacets,`program`,t,!0)}if(this.selectedTVShow){let e={key:this.selectedTVShow,count:0,state:`selected`};this.selectedFacets=He(this.selectedFacets,`program`,e)}}get tvDropdownFiltersTemplate(){if(this.searchType!==w.TV)return n;let{channelToNetwork:e,programToChannels:t}=this.dataSource.tvChannelMaps,r=e?[...new Set(e.values())]:[],i=t?[...t.entries()]:[];e&&this.selectedTVNetwork&&(i=i.filter(([,t])=>Object.keys(t).some(t=>e.get(t)===this.selectedTVNetwork)));let s=o(`Filter by Network`),c=o(`Filter by Show`),l=i.map(([e])=>e),u=a`
      <span slot="empty-options">
        <img src="https://archive.org/images/loading.gif" />
      </span>
    `,d=a`
      <span slot="empty-options">
        ${o(`Unable to fetch options, try again later`)}
      </span>
    `;return a`
      <div id="tv-filters" slot="facets-top">
        <ia-combo-box
          id="tv-networks"
          class="tv-filter-dropdown"
          placeholder=${s}
          clearable
          wrap-arrow-keys
          sort
          .options=${r.map((e,t)=>({id:`network-${t}`,text:e}))}
          @toggle=${this.tvDropdownToggled}
          @change=${this.networksDropdownChanged}
        >
          <span slot="label" class="sr-only">${s}</span>
          ${this.tvMapsLoading?u:n}
          ${this.tvMapsErrored?d:n}
        </ia-combo-box>
        <ia-combo-box
          id="tv-shows"
          class="tv-filter-dropdown"
          placeholder=${c}
          max-autocomplete-entries="500"
          clearable
          wrap-arrow-keys
          sort
          .options=${l.map((e,t)=>({id:`show-${t}`,text:e}))}
          @toggle=${this.tvDropdownToggled}
          @change=${this.showsDropdownChanged}
        >
          <span slot="label" class="sr-only">${c}</span>
          ${this.tvMapsLoading?u:n}
          ${this.tvMapsErrored?d:n}
        </ia-combo-box>
      </div>
    `}get facetsTemplate(){if(Re.includes(this.profileElement))return n;if(this.facetLoadStrategy===`off`)return a`
        <p class="facets-message">
          ${o(`Facets are temporarily unavailable.`)}
        </p>
      `;let e=this.isTVCollection||!this.withinCollection&&this.searchType===w.TV,t=e?Te:we,r=a`
      <collection-facets
        .collectionPagePath=${this.collectionPagePath}
        .parentCollections=${this.dataSource.parentCollections}
        .pageSpecifierParams=${this.dataSource.pageSpecifierParams}
        .searchService=${this.searchService}
        .featureFeedbackService=${this.featureFeedbackService}
        .recaptchaManager=${this.recaptchaManager}
        .resizeObserver=${this.resizeObserver}
        .searchType=${this.searchType}
        .aggregations=${this.dataSource.aggregations}
        .histogramAggregation=${this.dataSource.histogramAggregation}
        .minSelectedDate=${this.minSelectedDate}
        .maxSelectedDate=${this.maxSelectedDate}
        .selectedFacets=${this.selectedFacets}
        .baseNavigationUrl=${this.baseNavigationUrl}
        .collectionTitles=${this.dataSource.collectionTitles}
        .tvChannelAliases=${this.dataSource.tvChannelAliases}
        .showHistogramDatePicker=${this.showHistogramDatePicker}
        .allowExpandingDatePicker=${!this.mobileView}
        .allowDatePickerMonths=${e}
        .contentWidth=${this.contentWidth}
        .query=${this.baseQuery}
        .identifiers=${this.identifiers}
        .filterMap=${this.dataSource.filterMap}
        .isManageView=${this.isManageView}
        .modalManager=${this.modalManager}
        .analyticsHandler=${this.analyticsHandler}
        .facetDisplayOrder=${t}
        .isTvSearch=${e}
        ?collapsableFacets=${this.mobileView}
        ?facetsLoading=${this.facetsLoading}
        ?histogramAggregationLoading=${this.facetsLoading}
        ?suppressMediatypeFacets=${this.suppressMediatypeFacets}
        @facetClick=${this.facetClickHandler}
        @facetsChanged=${this.facetsChanged}
        @histogramDateRangeUpdated=${this.histogramDateRangeUpdated}
      >
        ${this.tvDropdownFiltersTemplate}
      </collection-facets>
    `;return(this.facetLoadStrategy===`opt-in`||this.facetLoadStrategy===`opt-in-or-login`&&!this.loggedIn)&&!this.mobileView?a`
        <details
          class="desktop-facets-dropdown"
          @toggle=${e=>{let t=e.target;this.collapsibleFacetsVisible=t.open}}
        >
          <summary>
            <span class="collapser-icon">${Je}</span>
            <h2>${o(`Filters`)}</h2>
          </summary>
          ${r}
        </details>
      `:r}clearFiltersBtnTemplate(e){if(!this.hasActiveFilters)return n;let t=g({"clear-filters-btn":!0,mobile:e}),r=o(e?`Clear all`:`Clear all filters`);return a`
      <div class="clear-filters-btn-row">
        ${e?a`<span class="clear-filters-btn-separator">&nbsp;</span>`:n}
        <button class=${t} @click=${this.clearFilters}>
          ${r}
        </button>
      </div>
    `}get listHeaderTemplate(){return a`
      <div id="list-header">
        <tile-dispatcher
          .tileDisplayMode=${`list-header`}
          .resizeObserver=${this.resizeObserver}
          .sortParam=${this.sortParam}
          .defaultSortParam=${this.defaultSortParam}
          .mobileBreakpoint=${this.mobileBreakpoint}
          .loggedIn=${this.loggedIn}
          .suppressBlurring=${this.shouldSuppressTileBlurring}
          .tileActions=${this.tileActions}
        >
        </tile-dispatcher>
      </div>
    `}histogramDateRangeUpdated(e){let{minDate:t,maxDate:n}=e.detail;[this.minSelectedDate,this.maxSelectedDate]=[t,n],this.analyticsHandler?.sendEvent({category:this.searchContext,action:A.histogramChanged,label:this.dataSource.dateRangeQueryClause})}emitManageModeChangedEvent(){this.dispatchEvent(new CustomEvent(`manageModeChanged`,{detail:this.isManageView}))}async installDataSourceAndQueryState(e,t){this.dataSource&&this.removeController(this.dataSource),this.dataSource=e,this.addController(this.dataSource),this.baseQuery=t.baseQuery,this.profileElement=t.profileElement,this.searchType=t.searchType,this.selectedFacets=t.selectedFacets??O(),this.internalFilters=t.internalFilters,this.minSelectedDate=t.minSelectedDate,this.maxSelectedDate=t.maxSelectedDate,this.selectedSort=t.selectedSort??E.default,this.sortDirection=t.sortDirection,this.selectedTitleFilter=t.selectedTitleFilter,this.selectedCreatorFilter=t.selectedCreatorFilter,this.pagesToRender=this.initialPageNumber,this.dataSourceInstallInProgress=!0,this.requestUpdate(),await this.updateComplete,this.dataSourceInstallInProgress=!1,this.searchResultsLoading||(this.setTotalResultCount(this.dataSource.totalResults),this.setTileCount(this.dataSource.size)),this.refreshVisibleResults()}firstUpdated(){this.restoreState(),this.setInitialSize()}setInitialSize(){this.contentWidth=this.contentContainer.getBoundingClientRect().width,this.mobileView=this.contentWidth>0&&this.contentWidth<this.mobileBreakpoint,this.sendLayoutSizeAnalytics()}sendLayoutSizeAnalytics(){this.analyticsHandler&&(this.layoutSizeAnalyticsSent=!0,this.analyticsHandler.sendEvent({category:this.searchContext,action:this.mobileView?A.loadMobileView:A.loadDesktopView}))}updated(e){if(e.has(`placeholderType`)&&this.placeholderType===null&&(this.leftColIntersectionObserver||this.setupLeftColumnScrollListeners(),this.facetsIntersectionObserver||this.setupFacetsScrollListeners(),this.updateLeftColumnHeight()),e.has(`analyticsHandler`)&&!this.layoutSizeAnalyticsSent&&this.sendLayoutSizeAnalytics(),(e.has(`displayMode`)||e.has(`baseNavigationUrl`)||e.has(`baseImageUrl`)||e.has(`loggedIn`)||e.has(`tileActions`)||e.has(`tileLayoutType`))&&this.infiniteScroller?.reload(),e.has(`profileElement`)&&this.applyDefaultProfileSort(),e.has(`withinCollection`)&&this.withinCollection&&(this.baseQuery||(this.defaultSortField=this.withinCollection.startsWith(`fav-`)?E.datefavorited:E.weeklyview,this.defaultSortDirection=`desc`)),e.has(`baseQuery`)&&this.emitBaseQueryChanged(),e.has(`searchType`)&&this.emitSearchTypeChanged(),(e.has(`currentPage`)||e.has(`displayMode`))&&this.persistState(),(e.has(`baseQuery`)||e.has(`identifiers`)||e.has(`minSelectedDate`)||e.has(`maxSelectedDate`)||e.has(`selectedFacets`)||e.has(`searchService`)||e.has(`withinCollection`)||e.has(`withinProfile`)||e.has(`profileElement`))&&this.dataSource.refreshLetterCounts(),e.has(`selectedSort`)||e.has(`sortDirection`)){let t=e.get(`sortDirection`);this.sendSortByAnalytics(t),this.selectedSortChanged()}if(e.has(`selectedTitleFilter`)&&this.sendFilterByTitleAnalytics(e.get(`selectedTitleFilter`)),e.has(`selectedCreatorFilter`)&&this.sendFilterByCreatorAnalytics(e.get(`selectedCreatorFilter`)),this.updateFacetReadiness(),(e.has(`baseQuery`)||e.has(`identifiers`)||e.has(`searchType`)||e.has(`selectedTitleFilter`)||e.has(`selectedCreatorFilter`)||e.has(`minSelectedDate`)||e.has(`maxSelectedDate`)||e.has(`selectedSort`)||e.has(`sortDirection`)||e.has(`selectedFacets`)||e.has(`searchService`)||e.has(`withinCollection`)||e.has(`withinProfile`)||e.has(`profileElement`))&&this.handleQueryChange(),e.has(`searchResultsLoading`)&&this.emitSearchResultsLoadingChanged(),e.has(`facetsLoading`)&&this.facetsLoading&&this.collectionFacets&&(this.collectionFacets.moreLinksVisible=this.searchType!==w.FULLTEXT),e.has(`pagesToRender`)&&!this.dataSource.endOfDataReached&&this.infiniteScroller&&(this.infiniteScroller.itemCount=this.estimatedTileCount),e.has(`isManageView`)&&(this.isManageView?(this.displayMode=`grid`,this.fetchManageableSearchResults()):this.pageContext===`search`&&this.infiniteScroller?.reload(),this.infiniteScroller?.refreshAllVisibleCells(),e.get(`isManageView`)!==void 0&&this.emitManageModeChangedEvent()),e.has(`resizeObserver`)){let t=e.get(`resizeObserver`);t&&this.disconnectResizeObserver(t),this.setupResizeObserver()}this.ensureAvailableTilesDisplayed()}connectedCallback(){super.connectedCallback?.(),this.setupStateRestorationObserver(),this.setupResizeObserver()}disconnectedCallback(){this.resizeObserver&&this.disconnectResizeObserver(this.resizeObserver),this.boundNavigationHandler&&window.removeEventListener(`popstate`,this.boundNavigationHandler),this.deferredFetchTimer&&(window.clearTimeout(this.deferredFetchTimer),this.deferredFetchTimer=0),this.leftColIntersectionObserver?.disconnect(),this.facetsIntersectionObserver?.disconnect(),window.removeEventListener(`resize`,this.updateLeftColumnHeight)}handleResize(e){let t=this.mobileView;e.target===this.contentContainer&&(this.contentWidth=e.contentRect.width,this.mobileView=this.contentWidth>0&&this.contentWidth<this.mobileBreakpoint,this.mobileView&&!t&&(this.isResizeToMobile=!0)),this.updateLeftColumnHeight()}ensureAvailableTilesDisplayed(){this.infiniteScroller&&this.infiniteScroller.itemCount<this.dataSource.size&&this.setTileCount(this.dataSource.endOfDataReached?this.dataSource.size:this.estimatedTileCount)}updateFacetReadiness(){let e=this.collapsibleFacetsVisible||this.facetLoadStrategy===`opt-in-or-login`&&this.loggedIn,t=[`opt-in`,`opt-in-or-login`].includes(this.facetLoadStrategy),n=!this.mobileView&&(!t||e),r=this.mobileView&&e;this.dataSource.handleFacetReadinessChange(n||r)}setupLeftColumnScrollListeners(){let e=this.shadowRoot?.querySelector(`#left-column-scroll-sentinel`);e&&(this.leftColIntersectionObserver=new IntersectionObserver(this.updateLeftColumnHeight,{threshold:[...Array(201).keys()].map(e=>e/200)}),this.leftColIntersectionObserver.observe(e)),window.addEventListener(`resize`,this.updateLeftColumnHeight)}setupFacetsScrollListeners(){let e=this.shadowRoot?.querySelector(`#facets-scroll-sentinel`);e&&(this.facetsIntersectionObserver=new IntersectionObserver(this.updateFacetFadeOut),this.facetsIntersectionObserver.observe(e))}emitBaseQueryChanged(){this.dispatchEvent(new CustomEvent(`baseQueryChanged`,{detail:{baseQuery:this.baseQuery}}))}emitSearchTypeChanged(){this.dispatchEvent(new CustomEvent(`searchTypeChanged`,{detail:this.searchType}))}emitFacetPaneVisibilityChanged(){this.dispatchEvent(new CustomEvent(`facetPaneVisibilityChanged`,{detail:this.facetPaneVisible}))}emitSearchError(){this.dispatchEvent(new CustomEvent(`searchError`,{detail:this.dataSource.queryErrorMessage}))}emitQueryStateChanged(){this.dispatchEvent(new CustomEvent(`queryStateChanged`,{detail:{baseQuery:this.baseQuery,withinCollection:this.withinCollection,withinProfile:this.withinProfile,profileElement:this.profileElement,searchType:this.searchType,selectedFacets:this.selectedFacets,internalFilters:this.internalFilters,minSelectedDate:this.minSelectedDate,maxSelectedDate:this.maxSelectedDate,selectedSort:this.selectedSort,sortDirection:this.sortDirection,selectedTitleFilter:this.selectedTitleFilter,selectedCreatorFilter:this.selectedCreatorFilter}}))}emitEmptyResults(){this.dispatchEvent(new Event(`emptyResults`))}disconnectResizeObserver(e){e.removeObserver({target:this.contentContainer,handler:this})}setupResizeObserver(){!this.resizeObserver||!this.contentContainer||this.resizeObserver.addObserver({target:this.contentContainer,handler:this})}visibleCellsChanged(e){this.updateVisiblePage(e.detail.visibleCellIndices)}updateVisiblePage(e){if(this.isScrollingToCell||e.length===0)return;let t=[...e].sort((e,t)=>e-t),n=t[Math.min(this.pageSize,t.length)-1],r=Math.floor(n/this.pageSize)+1;this.setCurrentPage(r)}setCurrentPage(e){this.currentPage!==e&&(this.currentPage=e,this.dispatchEvent(new CustomEvent(`visiblePageChanged`,{detail:{pageNumber:e}})))}get initialSearchComplete(){return this.dataSource.initialSearchComplete}async handleQueryChange(){!this.searchService||this.dataSource.pageFetchQueryKey===this.previousQueryKey||this.baseQuery&&!this.dataSource.canPerformSearch||(this.previousQueryKey=this.dataSource.pageFetchQueryKey,this.totalResults=void 0,this.pagesToRender=this.initialPageNumber===1?2:this.initialPageNumber,this.infiniteScroller&&(this.infiniteScroller.itemCount=this.estimatedTileCount,this.infiniteScroller.reload()),this.withinCollection&&this.baseQuery?.trim()&&(this.defaultSortField=E.relevance,this.defaultSortDirection=null),!this.initialQueryChangeHappened&&this.initialPageNumber>1&&this.scrollToPage(this.initialPageNumber),this.initialQueryChangeHappened=!0,this.historyPopOccurred||this.persistState(),this.historyPopOccurred=!1)}setupStateRestorationObserver(){this.boundNavigationHandler||(this.boundNavigationHandler=this.historyNavigationHandler.bind(this)),window.addEventListener(`popstate`,this.boundNavigationHandler)}historyNavigationHandler(){this.historyPopOccurred=!0,this.restoreState()}restoreState(){let e=this.restorationStateHandler.getRestorationState();this.displayMode=e.displayMode,!this.suppressURLSinParam&&e.searchType!=null&&(this.searchType=e.searchType),this.selectedSort=e.selectedSort??E.default,this.sortDirection=e.sortDirection??null,this.selectedTitleFilter=e.selectedTitleFilter??null,this.selectedCreatorFilter=e.selectedCreatorFilter??null,this.selectedFacets=e.selectedFacets,this.suppressURLQuery||(this.baseQuery=e.baseQuery),this.setCurrentPage(e.currentPage??1),this.minSelectedDate=e.minSelectedDate,this.maxSelectedDate=e.maxSelectedDate,this.currentPage&&this.currentPage>1&&this.goToPage(this.currentPage)}persistState(){let e=this.selectedSort===this.defaultSortField,t={displayMode:this.displayMode,searchType:this.suppressURLSinParam?void 0:this.searchType,selectedSort:e?E.default:this.selectedSort,sortDirection:this.sortDirection??void 0,selectedFacets:this.selectedFacets??O(),baseQuery:this.suppressURLQuery?void 0:this.baseQuery,currentPage:this.currentPage,titleQuery:this.titleQuery,creatorQuery:this.creatorQuery,minSelectedDate:this.minSelectedDate,maxSelectedDate:this.maxSelectedDate,selectedTitleFilter:this.selectedTitleFilter??void 0,selectedCreatorFilter:this.selectedCreatorFilter??void 0},n={forceReplace:this.dataSourceInstallInProgress,persistMetadataSearchType:this.isTVCollection};this.restorationStateHandler.persistState(t,n)}emitSearchResultsLoadingChanged(){this.dispatchEvent(new CustomEvent(`searchResultsLoadingChanged`,{detail:{loading:this.searchResultsLoading}}))}facetsChanged(e){this.selectedFacets=e.detail}facetClickHandler({detail:{facetType:e,bucket:t,negative:n}}){let r;r=n?t.state===`none`?A.facetNegativeDeselected:A.facetNegativeSelected:t.state===`none`?A.facetDeselected:A.facetSelected,this.analyticsHandler?.sendEvent({category:this.searchContext,action:r,label:e})}async scrollToPage(e){let t=this.pageSize*(e-1),n=0;for(;!this.infiniteScroller&&n<20;)await this.updateComplete,n++;if(!this.infiniteScroller)return;this.infiniteScroller.itemCount<this.estimatedTileCount&&(this.infiniteScroller.itemCount=this.estimatedTileCount,await this.updateComplete),await new Promise(e=>{setTimeout(e,0)}),this.isScrollingToCell=!0;let r=await this.infiniteScroller.scrollToCell(t,!0);this.isScrollingToCell=!1,this.infiniteScroller.refreshAllVisibleCells(),r&&this.updateVisiblePage(this.infiniteScroller.getVisibleCellIndices())}get isRelevanceSortAvailable(){return!!this.baseQuery?.trim()}setTileCount(e){this.infiniteScroller&&(this.infiniteScroller.itemCount=e)}applyDefaultTVSearchSort(){this.defaultSortField=E.datearchived,this.defaultSortDirection=`desc`}applyDefaultCollectionSort(e){if(this.baseQuery){this.defaultSortField=E.relevance,this.defaultSortDirection=null;return}let t=e?.public_metadata?.identifier?.startsWith(`fav-`)?`-favoritedate`:`-week`,[n,r]=(e?.public_metadata?.[`sort-by`]??t).split(`:`);n.startsWith(`-`)?(n=n.slice(1),r=`desc`):[`asc`,`desc`].includes(r)||(r=`asc`);let i=_e(n).field;i&&i!==E.default&&(this.defaultSortField=i,this.defaultSortDirection=r)}applyDefaultProfileSort(){if(this.profileElement){let e=xe[this.profileElement];this.defaultSortField=e??E.weeklyview}else this.defaultSortField=E.weeklyview;this.defaultSortDirection=`desc`}get currentVisiblePageNumbers(){let e=this.infiniteScroller?.getVisibleCellIndices()??[],t=new Set;return e.forEach(e=>{let n=Math.floor(e/this.pageSize)+1;t.add(n)}),Array.from(t)}get isRadioCollection(){let{withinCollection:e}=this,t=[`radio`],n=t.includes(e),r=t.some(e=>this.dataSource.parentCollections?.includes(e));return n||r}refreshVisibleResults(){this.infiniteScroller?.refreshAllVisibleCells()}resultSelected(e){if(this.isManageView){let t=this.dataSource.indexOf(e.detail);t>=0&&this.infiniteScroller?.refreshCell(t),this.requestUpdate()}this.analyticsHandler?.sendEvent({category:this.searchContext,action:A.resultSelected,label:e.detail.mediatype}),this.analyticsHandler?.sendEvent({category:this.searchContext,action:A.resultSelected,label:`page-${this.currentPage}`})}cellForIndex(e){let t=this.tileModelAtCellIndex(e);if(!t)return;let n=this.searchType===w.TV,r=this.searchType===w.RADIO,{isTVCollection:i,isRadioCollection:o}=this,s=n||r||i||o;return a`
      <tile-dispatcher
        .collectionPagePath=${this.collectionPagePath}
        .baseNavigationUrl=${this.baseNavigationUrl}
        .baseImageUrl=${this.baseImageUrl}
        .model=${t}
        .tileDisplayMode=${this.displayMode}
        .resizeObserver=${this.resizeObserver}
        .collectionTitles=${this.dataSource.collectionTitles}
        .sortParam=${this.sortParam}
        .defaultSortParam=${this.defaultSortParam}
        .creatorFilter=${this.selectedCreatorFilter}
        .mobileBreakpoint=${this.mobileBreakpoint}
        .loggedIn=${this.loggedIn}
        .suppressBlurring=${this.shouldSuppressTileBlurring}
        .isManageView=${this.isManageView}
        .tileActions=${this.tileActions}
        .layoutType=${this.tileLayoutType}
        ?showTvClips=${n||i}
        ?enableHoverPane=${!0}
        ?useLocalTime=${s}
        @resultSelected=${e=>this.resultSelected(e)}
      >
      </tile-dispatcher>
    `}scrollThresholdReached(){!this.dataSource.endOfDataReached&&this.dataSource.queryInitialized&&(this.pagesToRender+=1,this.dataSource.fetchPage(this.pagesToRender))}fetchManageableSearchResults(){let e=!this.dataSource.totalResults,t=!this.searchResultsLoading&&this.dataSource.totalResults>100;this.pageContext===`search`&&(e||t)&&(this.dataSource.resetPages(),this.dataSource.fetchPage(1,this.maxPagesToManage),this.infiniteScroller?.reload())}static get styles(){return[j,e`
        :host {
          display: block;
          --leftColumnWidth: 18rem;
          --leftColumnPaddingTop: 2rem;
          --leftColumnPaddingRight: 2.5rem;
        }

        #facet-top-view {
          display: flex;
        }

        /**
        * When page width resizes from desktop to mobile, use this class to
        * disable expand/collapse transition when loading.
        */
        .preload * {
          transition: none !important;
          -webkit-transition: none !important;
          -moz-transition: none !important;
          -ms-transition: none !important;
          -o-transition: none !important;
        }

        #content-container {
          display: flex;
        }

        empty-placeholder {
          margin-top: var(--placeholderMarginTop, 0);
        }

        .collapser-icon {
          display: inline-block;
        }

        .collapser-icon svg {
          display: inline-block;
          width: 12px;
          height: 12px;
          transition: transform 0.2s ease-out;
        }

        #mobile-filter-collapse {
          width: 100%;
        }

        #mobile-filter-collapse > summary {
          cursor: pointer;
          list-style: none;
        }

        #mobile-filter-collapse[open] > summary {
          margin-bottom: 10px;
        }

        #mobile-filter-collapse h2 {
          display: inline-block;
          margin: 0;
          font-size: 2rem;
        }

        #mobile-filter-collapse[open] svg {
          transform: rotate(90deg);
        }

        #content-container.mobile {
          display: block;
        }

        #right-column {
          flex: 1;
          position: relative;
          min-height: 90vh;
          border-right: 1px solid rgb(232, 232, 232);
          margin-top: var(--rightColumnMarginTop, 0);
          padding-top: var(--rightColumnPaddingTop, 2rem);
          background: #fff;
        }

        #left-column:not([hidden]) + #right-column {
          border-left: 1px solid rgb(232, 232, 232);
        }

        #right-column.smart-results-spacing {
          padding-top: 0.5rem;
          border-right: none;
          background: transparent;
          min-width: 0;
        }

        #results {
          background: #fff;
          padding-left: 1rem;
          padding-right: 1rem;
        }

        #right-column.smart-results-spacing #results {
          border-radius: 10px 10px 0px 0px;
          padding-top: 0.5rem;
          margin-top: 1rem;
        }

        .mobile #right-column {
          border-left: none;
        }

        .mobile #results {
          padding: 5px 5px 0;
        }

        /*
         * The results sit in a row with the optional results-aside slot, which
         * lays out beside them as a flex item and takes its own width and spacing.
         * The aside comes first in the DOM so keyboard and screen reader users
         * reach it before the infinitely scrolling tiles, but it shows on the right.
         */
        #results-body {
          display: flex;
          align-items: flex-start;
        }

        #results-main {
          order: -1;
          flex: 1;
          min-width: 0;
        }

        #left-column {
          width: var(--leftColumnWidth, 18rem);
          /* Prevents Safari from shrinking col at first draw */
          min-width: var(--leftColumnWidth, 18rem);
          padding-top: var(--leftColumnPaddingTop, 2rem);
          /* Reduced padding by 0.2rem to add the invisible border in the rule below */
          padding-right: calc(var(--leftColumnPaddingRight, 2.5rem) - 0.2rem);
          border-right: 0.2rem solid transparent; /* Pads to the right of the scrollbar a bit */
          z-index: 1;
        }

        .desktop #left-column {
          top: 0;
          position: sticky;
          height: calc(100vh - 2rem);
          max-height: calc(100vh - 2rem);
          overflow-x: hidden;
          overflow-y: scroll;

          /*
          * Firefox doesn't support any of the -webkit-scrollbar stuff below, but
          * does at least give us a tiny bit of control over width & color.
          */
          scrollbar-width: thin;
          scrollbar-color: transparent transparent;
        }
        .desktop #left-column:hover {
          scrollbar-color: auto;
        }
        .desktop #left-column::-webkit-scrollbar {
          appearance: none;
          width: 6px;
        }
        .desktop #left-column::-webkit-scrollbar-button {
          height: 3px;
          background: transparent;
        }
        .desktop #left-column::-webkit-scrollbar-corner {
          background: transparent;
        }
        .desktop #left-column::-webkit-scrollbar-thumb {
          border-radius: 4px;
        }
        .desktop #left-column:hover::-webkit-scrollbar-thumb {
          background: rgba(0, 0, 0, 0.15);
        }
        .desktop #left-column:hover::-webkit-scrollbar-thumb:hover {
          background: rgba(0, 0, 0, 0.2);
        }
        .desktop #left-column:hover::-webkit-scrollbar-thumb:active {
          background: rgba(0, 0, 0, 0.3);
        }

        #facets-bottom-fade {
          background: linear-gradient(
            to bottom,
            #fbfbfd00 0%,
            #fbfbfdc0 50%,
            #fbfbfd 80%,
            #fbfbfd 100%
          );
          position: fixed;
          bottom: 0;
          height: 50px;
          /* Wide enough to cover the content, but leave the scrollbar uncovered */
          width: calc(
            var(--leftColumnWidth) + var(--leftColumnPaddingRight) - 10px
          );
          z-index: 2;
          pointer-events: none;
          transition: height 0.1s ease;
        }
        #facets-bottom-fade.hidden {
          height: 0;
        }

        .facets-message {
          font-size: 1.4rem;
        }

        .desktop-facets-dropdown > summary {
          cursor: pointer;
          list-style: none;
        }

        .desktop-facets-dropdown h2 {
          display: inline-block;
          margin: 0;
          font-size: 1.6rem;
        }

        .desktop-facets-dropdown[open] > summary {
          margin-bottom: 10px;
        }

        .desktop-facets-dropdown[open] svg {
          transform: rotate(90deg);
        }

        .desktop #left-column-scroll-sentinel {
          width: 1px;
          height: 100vh;
          background: transparent;
        }

        .desktop #facets-scroll-sentinel {
          width: 1px;
          height: 1px;
          background: transparent;
        }

        #facets-header-container {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          clear: both;
        }

        .desktop #facets-header-container {
          flex-wrap: wrap;
        }

        .mobile #left-column {
          width: 100%;
          min-width: 0;
          padding: 5px 0;
          border: 0;
        }

        .clear-filters-btn-row {
          display: inline-block;
        }

        .desktop .clear-filters-btn-row {
          width: 100%;
        }

        .clear-filters-btn {
          display: inline-block;
          appearance: none;
          margin: 0;
          padding: 0;
          border: 0;
          background: none;
          color: var(--ia-theme-link-color);
          font-size: 1.4rem;
          font-family: inherit;
          cursor: pointer;
        }

        .clear-filters-btn:hover {
          text-decoration: underline;
        }

        .clear-filters-btn-separator {
          display: inline-block;
          margin-left: 5px;
          border-left: 1px solid #2c2c2c;
          font-size: 1.4rem;
          line-height: 1.3rem;
        }

        #tv-filters {
          margin-bottom: 15px;
        }

        #tv-shows {
          --comboBoxListWidth: 300px;
        }

        .tv-filter-dropdown {
          display: block;
          font-size: 14px;
          margin-left: 1px;
          margin-bottom: 5px;
        }

        .tv-filter-dropdown::part(combo-box) {
          outline-offset: 1px;
        }

        .tv-filter-dropdown::part(option) {
          line-height: 1.1;
          padding: 7px;
        }

        .tv-filter-dropdown::part(clear-button) {
          flex: 0 0 26px;
          --combo-box-clear-icon-size: 14px;
        }

        .tv-filter-dropdown::part(icon) {
          width: 1.4rem;
          height: 1.4rem;
        }

        #facets-container {
          position: relative;
          max-height: 0;
          transition: max-height 0.2s ease-in-out;
          z-index: 1;
          margin-top: var(--facetsContainerMarginTop, 3rem);
          padding-bottom: 2rem;
        }

        .desktop #facets-container {
          width: 18rem;
        }

        .mobile #facets-container {
          overflow: hidden;
          padding-bottom: 0;
          padding-left: 10px;
          padding-right: 10px;
        }

        #facets-container.expanded {
          max-height: 2000px;
        }

        .results-section-heading {
          margin: 0.5rem 0.3rem;
          font-size: 2rem;
          line-height: 25px;
        }

        #results-total {
          display: flex;
          align-items: baseline;
        }

        #results-total:not(.filtered) {
          padding-bottom: 2rem;
        }

        .mobile #results-total {
          position: absolute;
          right: 10px;
        }

        #big-results-count {
          font-size: 2.4rem;
          font-weight: 500;
          margin-right: 5px;
        }

        .mobile #big-results-count {
          font-size: 2rem;
        }

        #big-results-label {
          font-size: 1.4rem;
          font-weight: 200;
        }

        #list-header {
          max-height: 4.2rem;
        }

        .loading-cover {
          position: absolute;
          top: 0;
          left: 0;
          width: 100%;
          height: 100%;
          display: flex;
          justify-content: center;
          z-index: 1;
          padding-top: 50px;
        }

        #tile-blur-label {
          display: flex;
          align-items: center;
          column-gap: 5px;
        }

        #tile-blur-check {
          margin: 0 5px 0 0;
          width: 15px;
        }

        circular-activity-indicator {
          width: 30px;
          height: 30px;
        }

        sort-filter-bar {
          display: block;
          margin-bottom: 4rem;
        }

        infinite-scroller {
          display: block;
          --infiniteScrollerRowGap: var(--collectionBrowserRowGap, 1.7rem);
          --infiniteScrollerColGap: var(--collectionBrowserColGap, 1.7rem);
        }

        infinite-scroller.list-compact {
          --infiniteScrollerCellMinWidth: var(
            --collectionBrowserCellMinWidth,
            100%
          );
          --infiniteScrollerCellMinHeight: 45px; /* override infinite scroller component */
          --infiniteScrollerCellMaxHeight: 56px;
          --infiniteScrollerRowGap: 10px;
        }

        infinite-scroller.list-detail {
          --infiniteScrollerCellMinWidth: var(
            --collectionBrowserCellMinWidth,
            100%
          );
          --infiniteScrollerCellMinHeight: var(
            --collectionBrowserCellMinHeight,
            5rem
          );
          /*
            30px in spec, compensating for a -4px margin
            to align title with top of item image
            src/tiles/list/tile-list.ts
          */
          --infiniteScrollerRowGap: 34px;
        }

        .mobile infinite-scroller.list-detail {
          --infiniteScrollerRowGap: 24px;
        }

        infinite-scroller.grid {
          --infiniteScrollerCellMinWidth: var(
            --collectionBrowserCellMinWidth,
            17rem
          );
          --infiniteScrollerCellMaxWidth: var(
            --collectionBrowserCellMaxWidth,
            1fr
          );
        }

        /* Allow tiles to shrink a bit further at smaller viewport widths */
        @media screen and (max-width: 880px) {
          infinite-scroller.grid {
            --infiniteScrollerCellMinWidth: var(
              --collectionBrowserCellMinWidth,
              15rem
            );
          }
        }
        /* At very small widths, maintain a 2-tile layout as far as it can reasonably go */
        @media screen and (max-width: 360px) {
          infinite-scroller.grid {
            --infiniteScrollerCellMinWidth: var(
              --collectionBrowserCellMinWidth,
              12rem
            );
          }
        }

        infinite-scroller.hidden {
          display: none;
        }
      `]}},ba=ya,ya.DEFERRED_FETCH_DELAY_MS=150,ya);C([t({type:String})],$.prototype,`baseNavigationUrl`,void 0),C([t({type:String})],$.prototype,`baseImageUrl`,void 0),C([t({type:Object})],$.prototype,`searchService`,void 0),C([t({type:Number})],$.prototype,`searchType`,void 0),C([t({type:String})],$.prototype,`withinCollection`,void 0),C([t({type:String})],$.prototype,`withinProfile`,void 0),C([t({type:String})],$.prototype,`profileElement`,void 0),C([t({type:String})],$.prototype,`baseQuery`,void 0),C([t({type:Array})],$.prototype,`identifiers`,void 0),C([t({type:String})],$.prototype,`displayMode`,void 0),C([t({type:String})],$.prototype,`selectedSort`,void 0),C([t({type:String})],$.prototype,`selectedTitleFilter`,void 0),C([t({type:String})],$.prototype,`selectedCreatorFilter`,void 0),C([t({type:String})],$.prototype,`sortDirection`,void 0),C([t({type:String})],$.prototype,`defaultSortField`,void 0),C([t({type:String})],$.prototype,`defaultSortDirection`,void 0),C([t({type:Number})],$.prototype,`pageSize`,void 0),C([t({type:Number})],$.prototype,`currentPage`,void 0),C([t({type:String})],$.prototype,`minSelectedDate`,void 0),C([t({type:String})],$.prototype,`maxSelectedDate`,void 0),C([t({type:Object})],$.prototype,`selectedFacets`,void 0),C([t({type:Object})],$.prototype,`internalFilters`,void 0),C([t({type:Boolean})],$.prototype,`showSmartFacetBar`,void 0),C([t({type:String})],$.prototype,`smartFacetBarLabel`,void 0),C([t({type:Boolean})],$.prototype,`showHistogramDatePicker`,void 0),C([t({type:Boolean})],$.prototype,`suppressPlaceholders`,void 0),C([t({type:Boolean})],$.prototype,`suppressResultCount`,void 0),C([t({type:Boolean})],$.prototype,`suppressResultTiles`,void 0),C([t({type:Boolean})],$.prototype,`suppressURLQuery`,void 0),C([t({type:Boolean})],$.prototype,`suppressURLSinParam`,void 0),C([t({type:Boolean})],$.prototype,`suppressSortBar`,void 0),C([t({type:Boolean})],$.prototype,`suppressDisplayModes`,void 0),C([t({type:Boolean})],$.prototype,`suppressMediatypeFacets`,void 0),C([t({type:String})],$.prototype,`facetLoadStrategy`,void 0),C([t({type:Boolean,reflect:!0})],$.prototype,`facetPaneVisible`,void 0),C([t({type:Boolean})],$.prototype,`clearResultsOnEmptyQuery`,void 0),C([t({type:String})],$.prototype,`collectionPagePath`,void 0),C([t({type:String,reflect:!0})],$.prototype,`searchContext`,void 0),C([t({type:String})],$.prototype,`pageContext`,void 0),C([t({type:Object})],$.prototype,`restorationStateHandler`,void 0),C([t({type:Number})],$.prototype,`mobileBreakpoint`,void 0),C([t({type:Boolean})],$.prototype,`loggedIn`,void 0),C([t({type:Object})],$.prototype,`resizeObserver`,void 0),C([t({type:Object})],$.prototype,`modalManager`,void 0),C([t({type:Object})],$.prototype,`featureFeedbackService`,void 0),C([t({type:Object})],$.prototype,`recaptchaManager`,void 0),C([t({type:Boolean})],$.prototype,`isTVCollection`,void 0),C([t({type:Boolean})],$.prototype,`isManageView`,void 0),C([t({type:Array})],$.prototype,`tileActions`,void 0),C([t({type:String})],$.prototype,`tileLayoutType`,void 0),C([t({type:String})],$.prototype,`manageViewLabel`,void 0),C([t({type:Boolean})],$.prototype,`enableSortOptionsSlot`,void 0),C([t({type:Boolean,reflect:!0})],$.prototype,`showSmartResults`,void 0),C([t({type:Number})],$.prototype,`maxPagesToManage`,void 0),C([t({type:Object})],$.prototype,`dataSource`,void 0),C([s()],$.prototype,`pagesToRender`,void 0),C([s()],$.prototype,`searchResultsLoading`,void 0),C([s()],$.prototype,`facetsLoading`,void 0),C([s()],$.prototype,`totalResults`,void 0),C([s()],$.prototype,`mobileView`,void 0),C([s()],$.prototype,`tileBlurOverrideState`,void 0),C([s()],$.prototype,`collapsibleFacetsVisible`,void 0),C([s()],$.prototype,`contentWidth`,void 0),C([s()],$.prototype,`placeholderType`,void 0),C([s()],$.prototype,`selectedTVNetwork`,void 0),C([s()],$.prototype,`selectedTVShow`,void 0),C([s()],$.prototype,`tvMapsPopulated`,void 0),C([s()],$.prototype,`tvMapsLoading`,void 0),C([s()],$.prototype,`tvMapsErrored`,void 0),C([S(`#content-container`)],$.prototype,`contentContainer`,void 0),C([S(`#left-column`)],$.prototype,`leftColumn`,void 0),C([S(`collection-facets`)],$.prototype,`collectionFacets`,void 0),C([S(`manage-bar`)],$.prototype,`manageBar`,void 0),C([S(`smart-facet-bar`)],$.prototype,`smartFacetBar`,void 0),C([S(`#tv-networks`)],$.prototype,`tvNetworksDropdown`,void 0),C([S(`#tv-shows`)],$.prototype,`tvShowsDropdown`,void 0),C([t({type:Object,attribute:!1})],$.prototype,`analyticsHandler`,void 0),C([S(`infinite-scroller`)],$.prototype,`infiniteScroller`,void 0),$=ba=C([x(`collection-browser`),i()],$);export{ge as a,E as i,ua as n,O as o,qe as r,fa as t};