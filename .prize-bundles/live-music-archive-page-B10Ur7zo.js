const __vite__mapDeps=(i,m=__vite__mapDeps,d=(m.f||(m.f=["offshoot_assets/lma-player-dock-71YGSZvs.js","offshoot_assets/index-NFRVgqV2.js","offshoot_assets/rolldown-runtime-QTnfLwEv.js","offshoot_assets/vendor-shared-62wh2Xkx.js","offshoot_assets/vendor-topnav-Dcg-INpV.js","offshoot_assets/vendor-lit-MQ4DzmMc.js","offshoot_assets/analytics-handler-DK_L78yH.js","offshoot_assets/vendor-router-BJcn_WPh.js","offshoot_assets/logo-PtatNvE7.js","offshoot_assets/queue-move-target-DLVHU4zh.js","offshoot_assets/format-show-date-DvNvho4x.js","offshoot_assets/models-BSAzn-k3.js","offshoot_assets/show-urls-vhuWq7P2.js","offshoot_assets/z-tokens-C3UDJHq4.js","offshoot_assets/lma-show-more-surface-RIe1yqij.js","offshoot_assets/dist-CD3AhsrO.js","offshoot_assets/lma-show-tile-B26bhMa9.js","offshoot_assets/plural-count-CuSJx8jp.js","offshoot_assets/format-number-rdMiVR61.js","offshoot_assets/lma-overlay-Rrt46Gde.js","offshoot_assets/surface-stack-CLYdtZD7.js","offshoot_assets/lma-history-surface-u0r9Ya6t.js","offshoot_assets/lma-info-surface-chrome-Ei-AGWQk.js","offshoot_assets/lma-taper-section-surface-C5XeIC57.js","offshoot_assets/provenance-BO-TH8Ag.js","offshoot_assets/show-login-modal-C9u5aoLY.js","offshoot_assets/dist-BqHmxVdQ.js","offshoot_assets/login-page-BiUY-YWQ.js","offshoot_assets/ia-button-DYuVUpnO.js","offshoot_assets/ia-status-indicator-Bd61y7bd.js","offshoot_assets/ia-auth-options-DIzxHnZF.js","offshoot_assets/dist-CjVcNP8S.js","offshoot_assets/purify.es-RmFZhtLI.js","offshoot_assets/email-link-DbmrSSqK.js","offshoot_assets/ia-logo-gray-circle-BADeBCkf.js","offshoot_assets/third-party-auth-service-C371OWBh.js","offshoot_assets/log-Bvw5tZC3.js","offshoot_assets/ia-icon-ClrHaxRi.js","offshoot_assets/esm-CWeBglMZ.js"])))=>i.map(i=>d[i]);
import{A as e,C as t,E as n,F as r,N as i,O as a,P as o,S as s,T as c,c as l,f as u,l as d,m as f,p,s as ee,t as m,w as h,x as g}from"./vendor-lit-MQ4DzmMc.js";import{n as te}from"./vendor-topnav-Dcg-INpV.js";import{_,h as v,l as ne,t as y}from"./index-NFRVgqV2.js";import{t as b}from"./ia-clearable-text-input-CaSQalrP.js";import{o as re,r as x}from"./dist-P4PnKJsG.js";import{UserServiceErrorType as ie}from"./dist-CXM97COZ.js";import{i as ae}from"./esm-CWeBglMZ.js";import{t as S}from"./plural-count-CuSJx8jp.js";import{t as C}from"./basename-BvnL6MPN.js";import{t as oe}from"./logo-PtatNvE7.js";import{n as w,r as se,t as ce}from"./queue-move-target-DLVHU4zh.js";import{t as T}from"./format-show-date-DvNvho4x.js";import{a as le,c as E,d as D,f as O,h as k,i as ue,l as A,m as de,n as j,o as fe,p as pe,r as me,s as M,t as N,u as P}from"./models-BSAzn-k3.js";import{a as F,i as I,n as L,t as he}from"./show-urls-vhuWq7P2.js";import{t as R}from"./z-tokens-C3UDJHq4.js";import{n as ge,r as z,t as _e}from"./lma-show-tile-B26bhMa9.js";import{i as ve,r as B,t as ye}from"./provenance-BO-TH8Ag.js";import{t as be}from"./surface-stack-CLYdtZD7.js";var xe=`/offshoot_assets/concert-browser-entry-Bf_-YFOs.png`,Se=class{constructor(e){this.wikipediaService=e,this.cache=new Map}about(e){let t=this.cache.get(e);if(t)return t;let n=this.lookUp(e);return this.cache.set(e,n),n}async lookUp(e){try{let t=await(await this.wikipediaService()).fetchArticleData(e),n=t?.extract?.preparedHTML,r=t?.url;return!n||!r?M:{status:`loaded`,extractHTML:n,articleUrl:r}}catch{return M}}},Ce=class{constructor(e,t={}){this.cache=new Map,this.searchService=e,this.primerAggregationSize=t.primerAggregationSize??1e4}primeCounts(){return this.primer||(this.primer=this.runPrimer().catch(()=>{this.primer=void 0})),this.primer}async get(e){let t=new Map;if(e.length===0)return t;await this.primeCounts();for(let n of e){let e=this.cache.get(n);e!==void 0&&t.set(n,e)}return t}async runPrimer(){let e={pageType:`collection_details`,pageTarget:E,filters:{mediatype:{etree:x.INCLUDE}},rows:0,aggregations:{simpleParams:[`collection`]},aggregationsSize:this.primerAggregationSize},t=await this.fetchResponseDetails(e);this.cacheCollectionBuckets(t)}cacheCollectionBuckets(e){let t=e.aggregations?.collection?.buckets??[];for(let e of t)typeof e!=`number`&&typeof e.key==`string`&&this.cache.set(e.key,e.doc_count)}async fetchResponseDetails(e){let t=await this.searchService.search(e);if(t.success)return t.success.response;throw t.error??Error(`Search failed without an error payload`)}};function we(e,t={}){let n=e.identifier;if(n)return{identifier:n,name:e.title?.value??n,showCount:null,imageUrl:z(n,t.imageBaseUrl)}}var Te=/^(.*?)\s*\blive\s+at\b\s*(.*)$/i,Ee=/(?:^|\s)on\s+\d{4}(?:-\d{1,2}(?:-\d{1,2})?)?\s*$/i,De=/\s+\d{4}-\d{1,2}(?:-\d{1,2})?\s*$/,Oe=/^\d{4}(?:-\d{1,2}(?:-\d{1,2})?)?$/;function ke(e){let t=e.match(Te);if(!t)return{};let n={},r=t[1].trim();r&&(n.artist=r);let i=t[2].replace(Ee,``).replace(De,``).trim();return i&&!Oe.test(i)&&(n.venue=i),n}function Ae(e,t={}){let n=e.identifier;if(!n)return;let r=e.date?.value;if(!r)return;let i=e.year?.value;if(i===void 0)return;let a=e.title?.value||n,o=ke(a),s=e.creator?.value||t.fallbackArtist||o.artist;if(!s)return;let c=e.avg_rating?.value,l={identifier:n,title:a,artist:s,date:r.toISOString(),year:i,avgRating:c===void 0?null:Math.round(c*10)/10,numReviews:e.num_reviews?.value??0,source:e.source?.value??``,collections:e.collection?.values??[],imageUrl:z(n,t.imageBaseUrl)},u=je(e)??o.venue;u&&(l.venue=u);let d=e.publicdate?.value;return d&&(l.publicdate=d.toISOString()),l}function je(e){let t=e.rawMetadata?.fields?.venue;if(typeof t==`string`&&t.trim())return t.trim();if(Array.isArray(t)){let e=t[0];if(typeof e==`string`&&e.trim())return e.trim()}}var Me={downloads:{field:`downloads`,direction:`desc`},dateAsc:{field:`date`,direction:`asc`},dateDesc:{field:`date`,direction:`desc`},publicdateDesc:{field:`publicdate`,direction:`desc`},avgRatingDesc:{field:`avg_rating`,direction:`desc`},numReviewsDesc:{field:`num_reviews`,direction:`desc`},titleAsc:{field:`titleSorter`,direction:`asc`},creatorAsc:{field:`creatorSorter`,direction:`asc`}};function V(e){return[{...Me[e]}]}var H=[`identifier`,`title`,`creator`,`collection`,`date`,`year`,`avg_rating`,`num_reviews`,`source`,`publicdate`],Ne=[`identifier`,`title`],Pe=[31,29,31,30,31,30,31,31,30,31,30,31],Fe=4.5,Ie=3,Le=24,Re=24,ze=24,Be=12,Ve=12,He=24,Ue=`SearchService.SearchEngineError`,We=50,Ge=200,Ke=1800,qe=class{constructor(e,t={}){this.searchService=e,this.imageBaseUrl=t.imageBaseUrl}async searchShows(e={}){let{collection:t,query:n,sort:r=`downloads`,rows:i=Le,page:a=1,firstCreatorLetter:o,year:s}=e,c=this.showFilters();o&&(c.firstCreator={[o]:x.INCLUDE}),s!==void 0&&(c.year={[String(s)]:x.INCLUDE});let l={pageType:`collection_details`,pageTarget:t??`etree`,fields:[...H],filters:c,sort:V(r),rows:i,page:a,aggregations:{omit:!0}},u=n?.trim();return u&&(l.query=u),this.fetchShows(l,t)}async topShows(e={}){let{sort:t=`downloads`,rows:n=Re,page:r=1,firstCreatorLetter:i,year:a}=e;return this.searchShows({sort:t,rows:n,page:r,firstCreatorLetter:i,year:a})}async topRatedShows(e={}){let{rows:t=ze,page:n=1}=e,r=this.showFilters();r.avg_rating={[String(Fe)]:x.GREATER_OR_EQUAL},r.num_reviews={[String(Ie)]:x.GREATER_OR_EQUAL};let i={pageType:`collection_details`,pageTarget:E,fields:[...H],filters:r,sort:V(`downloads`),rows:t,page:n,aggregations:{omit:!0}};return this.fetchShows(i)}async freshlyUploaded(e={}){let{rows:t=Be,page:n=1}=e;return this.searchShows({sort:`publicdateDesc`,rows:t,page:n})}async onThisDay(e){let{month:t,day:n,rows:r=Ve}=e;if(!(Number.isInteger(t)&&t>=1&&t<=12&&Number.isInteger(n)&&n>=1&&n<=Pe[t-1]))throw RangeError(`Invalid calendar day: month ${t}, day ${n}`);let i={pageType:`collection_details`,pageTarget:E,query:`title:"-${String(t).padStart(2,`0`)}-${String(n).padStart(2,`0`)}"`,fields:[...H],filters:this.showFilters(),sort:V(`downloads`),rows:r,page:1,aggregations:{omit:!0}};return this.fetchShows(i)}async favoriteShows(e,t={}){let{rows:n=He,page:r=1}=t,i={pageType:`collection_details`,pageTarget:e,fields:[...H],filters:this.showFilters(),sort:V(`downloads`),rows:n,page:r,aggregations:{omit:!0}};try{return await this.fetchShows(i)}catch(t){if(t instanceof re&&t.type===Ue&&t.message.includes(`${e}: is not a collection`))return[];throw t}}async getArtistCollections(e={}){let{query:t,sort:n=`downloads`,firstLetter:r,rows:i=We,page:a=1}=e,o={mediatype:{collection:x.INCLUDE}};r&&(o.firstTitle={[r]:x.INCLUDE});let s={pageType:`collection_details`,pageTarget:E,fields:[...Ne],filters:o,sort:V(n),rows:i,page:a,aggregations:{omit:!0}},c=t?.trim();return c&&(s.query=c),(await this.fetchResponseDetails(s)).results.flatMap(e=>{let t=we(e,{imageBaseUrl:this.imageBaseUrl});return t?[t]:[]})}async showCount(e,t={}){let n=this.showFilters();t.alsoInCollection&&(n.collection={[t.alsoInCollection]:x.INCLUDE});let r={pageType:`collection_details`,pageTarget:e,fields:[`identifier`],filters:n,rows:0,aggregations:{omit:!0}};return(await this.fetchResponseDetails(r)).totalResults}async showYears(){let e={pageType:`collection_details`,pageTarget:E,fields:[`identifier`],filters:this.showFilters(),rows:0,aggregations:{simpleParams:[`year`]},aggregationsSize:Ge},t=(await this.fetchResponseDetails(e)).aggregations?.year?.buckets??[],n=new Date().getUTCFullYear()+1,r=t.flatMap(e=>{if(typeof e==`number`||e.doc_count<=0)return[];let t=String(e.key).trim();if(!t)return[];let r=Number(t);return Number.isInteger(r)&&r>=Ke&&r<=n?[r]:[]});return[...new Set(r)].sort((e,t)=>t-e)}showFilters(){return{mediatype:{etree:x.INCLUDE}}}async fetchResponseDetails(e){let t=await this.searchService.search(e);if(t.success)return t.success.response;throw t.error??Error(`Search failed without an error payload`)}async fetchShows(e,t){let n=await this.fetchResponseDetails(e),r=t?this.collectionDisplayTitle(n,t):void 0;return n.results.flatMap(e=>{let t=Ae(e,{fallbackArtist:r,imageBaseUrl:this.imageBaseUrl});return t?[t]:[]})}collectionDisplayTitle(e,t){let n=e.collectionTitles?.[t];if(n)return n;let r=e.collectionExtraInfo?.public_metadata?.title;return typeof r==`string`&&r?r:void 0}},Je=`https://archive.org`,Ye=`VBR MP3`,Xe=[`Flac`,`24bit Flac`,`Ogg Vorbis`,`WAVE`],Ze=[`JPEG`,`PNG`],Qe=class{constructor(e,t={}){this.detailsCache=new Map,this.metadataService=e,this.downloadBaseUrl=t.downloadBaseUrl??Je}async getShowDetails(e){let t=this.detailsCache.get(e);if(t)return t;let n=this.fetchShowDetails(e);return this.detailsCache.set(e,n),n.catch(()=>this.detailsCache.delete(e)),n}async fetchShowDetails(e){let t=await this.metadataService.fetchMetadata(e),n=t.success;if(!n)throw t.error??Error(`Metadata fetch failed without an error payload`);let r=n.files??[],i=n.metadata,a={setlist:this.buildSetlist(e,r),photos:this.originalPhotos(e,r),description:i.description?.values.join(`
`)??``,source:i.source?.value??``,lineage:i.lineage?.value??``,taper:i.taper?.value??``,transferer:i.transferer?.value??``,notes:i.notes?.values.join(`
`)??``,runtimeSecs:i.runtime?.value??null,addeddate:i.addeddate?.value?.toISOString()??``,collections:i.collection?.values??[]},o=i.venue?.value;return o&&(a.venue=o),a}buildSetlist(e,t){let n=this.playableFiles(t,[Ye]);if(n.length===0){for(let e of Xe)if(n=this.playableFiles(t,[e]),n.length>0)break}let r=[];for(let t of n){let n=t.length;n!==void 0&&r.push({title:t.title||this.nameWithoutExtension(t.name),fileName:t.name,durationSecs:n,url:this.downloadUrl(e,t.name)})}return r}playableFiles(e,t){let n=e.filter(e=>t.includes(e.format)&&e.length&&!e.private);return n.filter(e=>!this.isRedundantSample(e,n)).sort((e,t)=>this.discNumber(e.name)-this.discNumber(t.name)||(e.track??1/0)-(t.track??1/0)||e.name.localeCompare(t.name,void 0,{numeric:!0}))}isRedundantSample(e,t){let n=C(e.name).match(/^(.+)_sample\.mp3$/i)?.[1];if(n===void 0)return!1;let r=n.toLowerCase();return t.some(t=>t!==e&&this.nameWithoutExtension(t.name).toLowerCase()===r)}discNumber(e){let t=e.match(/(?:^|[^a-z])d(\d+)t\d/i)?.[1];return t?Number(t):1}originalPhotos(e,t){return t.filter(e=>Ze.includes(e.format)&&e.source===`original`).map(t=>this.downloadUrl(e,t.name))}downloadUrl(e,t){return te(e,t,this.downloadBaseUrl)}nameWithoutExtension(e){return C(e).replace(/\.[^.]+$/,``)}},$e=class{constructor(e){this.keyValueStoreNamespace=`LMAUserState`,this.eqPresetsKey=`eqPresets`,this.queueKey=`queue`,this.recentlyPlayedKey=`recentlyPlayed`,this.sectionOrderKey=`sectionOrder`,this.volumeKey=`volume`,this.defaultFavoriteMediatype=`etree`,this.eqBandCount=10,this.favoriteService=e.favoriteService,this.keyValueStore=e.keyValueStoreManager.getKeyValueStore({namespace:this.keyValueStoreNamespace})}async isFavorite(e,t){return this.resolveFavoriteCall(()=>this.favoriteService.checkFavorite({identifier:e,mediatype:t?.mediatype??this.defaultFavoriteMediatype}))}async favoritesAmong(e){return this.resolveFavoriteCall(()=>this.favoriteService.checkFavorites({identifiers:e}))}async addFavorite(e,t){return this.resolveFavoriteCall(()=>this.favoriteService.addFavorite({identifier:e,mediatype:t?.mediatype??this.defaultFavoriteMediatype,title:t?.title}))}async removeFavorite(e,t){return this.resolveFavoriteCall(()=>this.favoriteService.removeFavorite({identifier:e,mediatype:t?.mediatype??this.defaultFavoriteMediatype}))}async getEQPresets(){return this.readStoredPresets()}async saveEQPreset(e,t){let n=await this.readStoredPresets(),r=n.findIndex(t=>t.name===e);r>=0?n[r]={name:e,gains:t}:n.push({name:e,gains:t}),await this.keyValueStore.set(this.eqPresetsKey,n)}async deleteEQPreset(e){let t=await this.readStoredPresets(),n=t.filter(t=>t.name!==e);n.length!==t.length&&await this.keyValueStore.set(this.eqPresetsKey,n)}async loadQueue(){try{let e=await this.keyValueStore.get(this.queueKey);if(!e)return;let t=e.version===2||le.includes(e.version),n=Array.isArray(e.tracks)&&e.tracks.every(e=>this.queueTrackIsValid(e)),r=n&&Number.isInteger(e.currentTrackIndex)&&e.currentTrackIndex>=0&&e.currentTrackIndex<e.tracks.length;if(!t||!n||!r){await this.keyValueStore.delete(this.queueKey);return}return{...e,version:2}}catch{return}}async saveQueue(e){try{await this.keyValueStore.set(this.queueKey,e)}catch{}}async clearQueue(){try{await this.keyValueStore.delete(this.queueKey)}catch{}}async loadRecentlyPlayed(){let e;try{e=await this.keyValueStore.get(this.recentlyPlayedKey)}catch{return{status:`failed`}}if(!e)return{status:`empty`};let t=e.version===1,n=Array.isArray(e.tracks)&&e.tracks.every(e=>this.historyTrackIsValid(e));if(!t||!n){try{await this.keyValueStore.delete(this.recentlyPlayedKey)}catch{}return{status:`empty`}}return{status:`loaded`,history:{...e,version:1}}}async saveRecentlyPlayed(e){try{await this.keyValueStore.set(this.recentlyPlayedKey,e)}catch{}}async loadSectionOrder(){let e=await this.keyValueStore.get(this.sectionOrderKey);if(!e)return;if(!Array.isArray(e)){await this.keyValueStore.delete(this.sectionOrderKey);return}let t=e.filter(e=>this.sectionIdIsValid(e));return[...new Set(t)]}async saveSectionOrder(e){await this.keyValueStore.set(this.sectionOrderKey,e)}async loadVolume(){try{let e=await this.keyValueStore.get(this.volumeKey);if(e==null)return;if(!(typeof e==`number`&&Number.isFinite(e)&&e>=0&&e<=1)){await this.keyValueStore.delete(this.volumeKey);return}return e}catch{return}}async saveVolume(e){try{await this.keyValueStore.set(this.volumeKey,e)}catch{}}async resolveFavoriteCall(e){try{return await e()}catch(e){return{success:!1,error:String(e)}}}async readStoredPresets(){let e=await this.keyValueStore.get(this.eqPresetsKey);return Array.isArray(e)?e.filter(e=>this.presetIsValid(e)):[]}presetIsValid(e){let t=e?.gains;return typeof e?.name==`string`&&Array.isArray(t)&&t.length===this.eqBandCount&&t.every(e=>typeof e==`number`)}sectionIdIsValid(e){return typeof e==`string`&&O.includes(e)}queueTrackIsValid(e){return typeof e?.showIdentifier==`string`&&typeof e.fileName==`string`&&typeof e.trackTitle==`string`&&typeof e.durationSecs==`number`&&typeof e.showTitle==`string`&&typeof e.showDate==`string`&&typeof e.artist==`string`&&(e.played===void 0||typeof e.played==`boolean`)}historyTrackIsValid(e){return typeof e?.showIdentifier==`string`&&typeof e.fileName==`string`&&typeof e.trackTitle==`string`&&typeof e.durationSecs==`number`&&typeof e.trackUrl==`string`&&Number.isInteger(e.setlistIndex)&&e.setlistIndex>=0&&typeof e.showTitle==`string`&&typeof e.showDate==`string`&&typeof e.artist==`string`&&typeof e.sessionId==`string`&&(e.venue===void 0||typeof e.venue==`string`)&&(e.streamOnly===void 0||typeof e.streamOnly==`boolean`)}};function et(e){return Number.isFinite(e)?Math.min(1,Math.max(0,e)):0}function tt(){let e=globalThis.crypto?.randomUUID;return typeof e==`function`?e.call(globalThis.crypto):`${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`}function nt(e,t,n){let r=n===void 0?-1:e.findIndex(e=>e.fileName===n);return r>=0?r:t??0}function rt(e){return e>=1024?`desktop`:e>=640?`tablet`:`mobile`}var it=class{constructor(e){this.entries=[],this.currentIndex=-1,this.transitionListener=e}get current(){return this.entries[this.currentIndex]}get trackCount(){return this.entries.length-Math.max(0,this.currentIndex)}get showCount(){return new Set(this.entries.map(e=>e.show.identifier)).size}get playOrder(){return{entries:this.entries.map((e,t)=>({position:t,show:e.show,track:e.track,played:e.played})),currentPosition:this.currentIndex}}get hasPrevious(){let e=this.currentIndex;return e>0&&e>this.runStart(e)}get hasNext(){return this.currentIndex>=0&&this.currentIndex<this.entries.length-1}startShow(e,t,n){if(t.length===0)return;let r=e.identifier,i=new Map;this.entries.slice(0,this.currentIndex+1).forEach(e=>{e.show.identifier===r&&i.set(e.setlistIndex,e.played)});let a=t.map((t,n)=>({show:e,track:t,setlistIndex:n,played:i.get(n)??!1,ephemeral:!0})),o=t[n]?n:0;return this.playAtFront(e=>e.show.identifier===r,a,o)}startTrack(e,t,n){let r=e.identifier,i=this.current,a=[{show:e,track:t,setlistIndex:n,played:i?.show.identifier===r&&i.setlistIndex===n&&i.played,ephemeral:!0}],o=Math.max(this.currentIndex,0);return this.playAtFront((e,t)=>t>=o&&e.show.identifier===r&&e.setlistIndex===n,a,0)}appendShow(e,t){let n=t.map((e,t)=>({track:e,setlistIndex:t})).filter(({setlistIndex:t})=>!this.hasTrack(e.identifier,t));for(let{track:t,setlistIndex:r}of n)this.entries.push(this.queuedEntry(e,t,r));return n.length>0}appendTrack(e,t,n){return this.hasTrack(e.identifier,n)?!1:(this.entries.push(this.queuedEntry(e,t,n)),!0)}hasTrackFile(e,t){return this.currentAndUpcoming().some(n=>n.show.identifier===e&&n.track.fileName===t)}hasShow(e){return this.currentAndUpcoming().some(t=>t.show.identifier===e)}hasTrack(e,t){return this.currentAndUpcoming().some(n=>n.show.identifier===e&&n.setlistIndex===t)}startAtFirst(){return this.currentIndex<0&&this.entries.length>0?this.moveCurrentTo(0):this.current}markCurrentPlayed(){let e=this.current;if(!e||e.played)return!1;let t={...e,played:!0};return this.entries[this.currentIndex]=t,this.emit({kind:`trackPlayed`,entry:t}),!0}jumpTo(e){if(this.isPosition(e))return this.moveCurrentTo(e)}moveEntry(e,t){let n=this.isPosition(e)&&this.isPosition(t)?ce(e,t,this.currentIndex):void 0;if(n===void 0)return!1;let[r]=this.entries.splice(e,1);return this.entries.splice(n,0,r),e<this.currentIndex&&--this.currentIndex,this.currentIndex>=0&&this.clearPlayedAfter(this.currentIndex),!0}removeEntry(e){if(!this.isPosition(e))return{outcome:`ignored`};let t=this.currentIndex,[n]=this.entries.splice(e,1);return e===t&&n&&this.emit({kind:`trackLeft`,entry:n}),this.entries.length===0?this.settleRemovedCurrent(0):e<t?(this.currentIndex=t-1,{outcome:`kept`}):e>t?{outcome:`kept`}:this.settleRemovedCurrent(t)}removeRun(e){if(!this.isPosition(e))return{outcome:`ignored`};let t=this.runStart(e),n=this.runEnd(e),r=this.currentIndex,i=this.current,a=r>=t&&r<n;return a&&i&&this.emit({kind:`trackLeft`,entry:i}),this.entries.splice(t,n-t),a||this.entries.length===0?this.settleRemovedCurrent(t):(r>=n&&(this.currentIndex=r-(n-t)),{outcome:`kept`})}restore(e,t){if(!(Number.isInteger(t)&&t>=0&&t<e.length))return;this.entries=e.map(e=>e.ephemeral?{...e,ephemeral:!1}:e),this.currentIndex=t,this.clearPlayedAfter(t);let n=this.runStart(t);return this.entries.splice(0,n),this.currentIndex-=n,this.current}pullBack(e){let t=this.current,n=t?this.runStart(this.currentIndex):0;return t&&this.emit({kind:`trackLeft`,entry:t}),this.entries.splice(n,0,{...e,played:!1}),this.currentIndex=n,this.clearPlayedAfter(n),this.current}queuedEntry(e,t,n){return{show:e,track:t,setlistIndex:n,played:!1,ephemeral:!1}}currentAndUpcoming(){return this.entries.slice(Math.max(this.currentIndex,0))}playAtFront(e,t,n){let r=this.current,i=this.currentIndex,a=t[n],o=r!==void 0&&r.show.identifier===a?.show.identifier&&r.setlistIndex===a.setlistIndex,s=[],c=[],l=-2;this.entries.forEach((t,n)=>{if(e(t,n))return;let r=n<i||n===i&&t.played;if(!t.ephemeral&&!r){s.push(t);return}let a=c[c.length-1],o=this.entries[n-1],u=l===n-1&&o?.show.identifier===t.show.identifier;a&&u?a.push(t):c.push([t]),l=n});let u=t.map((e,t)=>{let r=(t<n||t===n&&o)&&e.played;return e.played===r&&e.ephemeral?e:{...e,played:r,ephemeral:!0}});this.entries=[...u,...s],this.currentIndex=n,r&&!o&&this.emit({kind:`trackLeft`,entry:r});for(let e of c)this.emit({kind:`runDropped`,entries:e});return this.current}isPosition(e){return Number.isInteger(e)&&e>=0&&e<this.entries.length}settleRemovedCurrent(e){if(this.entries.length===0)return this.currentIndex=-1,{outcome:`emptied`};this.currentIndex=Math.min(Math.max(e,0),this.entries.length-1),this.clearPlayedAfter(this.currentIndex-1),this.dropRunsBefore(this.runStart(this.currentIndex));let t=this.current;return t?{outcome:`replaced`,entry:t}:{outcome:`emptied`}}advancePast(e,t={}){let n=this.current;if(!n||n.track.url!==e)return{outcome:`desynced`};let r=this.currentIndex===this.entries.length-1;if(r&&!t.emptyAtEnd)return{outcome:`exhausted`};if(r)return this.emit({kind:`trackLeft`,entry:n}),this.currentIndex=-1,this.dropRunsBefore(this.entries.length),{outcome:`emptied`};let i=this.moveCurrentTo(this.currentIndex+1);return i?{outcome:`advanced`,entry:i}:{outcome:`desynced`}}stepBack(){if(!this.hasPrevious)return{outcome:`atStart`};let e=this.moveCurrentTo(this.currentIndex-1);return e?{outcome:`retreated`,entry:e}:{outcome:`atStart`}}moveCurrentTo(e){let t=this.currentIndex,n=this.current;return n&&e!==t&&this.emit({kind:`trackLeft`,entry:n}),e<t&&this.clearPlayedAfter(e-1),this.currentIndex=e,this.dropRunsBefore(this.runStart(e)),this.current}runStart(e){let t=this.entries[e]?.show.identifier,n=e;for(;n>0&&this.entries[n-1]?.show.identifier===t;)--n;return n}runEnd(e){let t=this.entries[e]?.show.identifier,n=e+1;for(;this.entries[n]?.show.identifier===t;)n+=1;return n}clearPlayedAfter(e){this.entries=this.entries.map((t,n)=>n>e&&t.played?{...t,played:!1}:t)}dropRunsBefore(e){if(e<=0)return;let t=this.entries.splice(0,e);this.currentIndex>=0&&(this.currentIndex-=e),this.emitDroppedRuns(t)}emitDroppedRuns(e){let t=[];for(let n of e){let e=t[t.length-1],r=e?.[0]?.show.identifier===n.show.identifier;e&&r?e.push(n):t.push([n])}for(let e of t)this.emit({kind:`runDropped`,entries:e})}emit(e){this.transitionListener?.(e)}},at=class{constructor(e){this.records=[],this.sessionId=e}get entries(){return this.records.map(({show:e,track:t,setlistIndex:n})=>({show:e,track:t,setlistIndex:n}))}get snapshot(){return[...this.records]}record(e){let t={show:e.show,track:e.track,setlistIndex:e.setlistIndex,ephemeral:e.ephemeral,sessionId:this.sessionId},n=w(t),r=this.records.filter(e=>w(e)!==n);this.records=[t,...r].slice(0,200)}hasPullable(e){return this.pullableIndex(e)>=0}takeNewestPullable(e){let t=this.pullableIndex(e),n=this.records[t];if(n)return this.records=this.records.filter((e,n)=>n!==t),{show:n.show,track:n.track,setlistIndex:n.setlistIndex,played:!1,ephemeral:n.ephemeral}}restore(e){let t=[...this.records],n=new Set(t.map(w));for(let r of e){if(t.length>=200)break;let e=w(r);n.has(e)||(n.add(e),t.push(r))}this.records=t}pullableIndex(e){return this.records.findIndex(t=>t.sessionId===this.sessionId&&!e(t))}};function ot(e){let t=e.entries;if(t.length===0)return;let n=t.map(({show:e,track:t,played:n})=>{let r={showIdentifier:e.identifier,fileName:t.fileName,trackTitle:t.title,durationSecs:t.durationSecs,showTitle:e.title,showDate:e.date,artist:e.artist,played:n};return e.venue!==void 0&&(r.venue=e.venue),e.streamOnly!==void 0&&(r.streamOnly=e.streamOnly),r}),r=e.currentPosition;return{version:2,currentTrackIndex:r>=0&&r<t.length?r:0,tracks:n}}function st(e,t,n){let r=new Date(e.showDate).getUTCFullYear(),i={identifier:e.showIdentifier,title:e.showTitle,artist:e.artist,date:e.showDate,year:Number.isNaN(r)?0:r,avgRating:null,numReviews:0,source:``,collections:t.collections,streamOnly:ye({collections:t.collections}),imageUrl:z(e.showIdentifier,n)},a=e.venue??t.venue;return a!==void 0&&(i.venue=a),i}function ct(e,t,n){let r=new Map,i=[],a;if(e.tracks.forEach((o,s)=>{let c=o.showIdentifier,l=t.get(c),u=l?.setlist.findIndex(e=>e.fileName===o.fileName)??-1,d=l?.setlist[u];if(!l||!d)return;let f=r.get(c)??st(o,l,n);r.set(c,f),s>=e.currentTrackIndex&&a===void 0&&(a=i.length);let p=o.played===!0;i.push({show:f,track:d,setlistIndex:u,played:p,ephemeral:!1})}),i.length!==0)return{entries:i,currentIndex:a??i.length-1}}function lt(e){return{version:1,tracks:e.map(({show:e,track:t,setlistIndex:n,sessionId:r})=>{let i={showIdentifier:e.identifier,fileName:t.fileName,trackTitle:t.title,durationSecs:t.durationSecs,trackUrl:t.url,setlistIndex:n,showTitle:e.title,showDate:e.date,artist:e.artist,sessionId:r};return e.venue!==void 0&&(i.venue=e.venue),e.streamOnly!==void 0&&(i.streamOnly=e.streamOnly),i})}}function ut(e,t){let n=new Map;return e.tracks.map(e=>{let r=e.showIdentifier,i=n.get(r)??dt(e,t);return n.set(r,i),{show:i,track:{title:e.trackTitle,fileName:e.fileName,durationSecs:e.durationSecs,url:e.trackUrl},setlistIndex:e.setlistIndex,ephemeral:!1,sessionId:e.sessionId}})}function dt(e,t){let n=new Date(e.showDate).getUTCFullYear(),r={identifier:e.showIdentifier,title:e.showTitle,artist:e.artist,date:e.showDate,year:Number.isNaN(n)?0:n,avgRating:null,numReviews:0,source:``,collections:[],imageUrl:z(e.showIdentifier,t)};return e.venue!==void 0&&(r.venue=e.venue),e.streamOnly!==void 0&&(r.streamOnly=e.streamOnly),r}function ft(e){return e instanceof DOMException?e.name===`AbortError`?`benign`:e.name===`NotAllowedError`?`blocked`:`failure`:`failure`}var pt=250;function mt(e,t){return e===void 0||t===void 0?e===t:e.mediaErrorCode===t.mediaErrorCode&&e.url===t.url}var ht=class{constructor(e=new Audio,t={}){this.listeners={playingChanged:new Set,timeChanged:new Set,durationChanged:new Set,bufferingChanged:new Set,errorChanged:new Set,playbackStarted:new Set,trackEnded:new Set},this.playing=!1,this.buffering=!1,this.playIntent=!1,this.pauseRequested=!1,this.pauseRetryUsed=!1,this.endedReported=!1,this.handleElementPlay=()=>{this.pauseRequested=!1,this.endedReported=!1,this.setPlaying(!0)},this.handleElementPause=()=>{this.setPlaying(!1),!(this.pauseRequested||this.audio.ended||!this.playIntent)&&(this.pauseRetryUsed||(this.clearPauseRetry(),this.pauseRetryTimer=window.setTimeout(()=>{this.pauseRetryTimer=void 0,this.playIntent&&!this.pauseRequested&&this.audio.paused&&!this.audio.ended&&(this.pauseRetryUsed=!0,this.attemptPlay())},this.pauseRetryDelayMs)))},this.handleElementTimeUpdate=()=>{this.emit(`timeChanged`,this.audio.currentTime)},this.handleElementDurationChange=()=>{this.emit(`durationChanged`,this.durationSecs)},this.handleElementDataWait=e=>{if(!this.playIntent)return;let t=!this.audio.paused&&this.audio.readyState>=HTMLMediaElement.HAVE_FUTURE_DATA;e.type===`stalled`&&t||this.setBuffering(!0)},this.handleElementDataReady=()=>{this.setBuffering(!1)},this.handleElementPlaying=()=>{this.setBuffering(!1),this.setError(void 0);let e=this.loadedUrl;e&&this.emit(`playbackStarted`,e)},this.handleElementError=()=>{this.setBuffering(!1),this.setError({mediaErrorCode:this.audio.error?.code,url:this.loadedUrl})},this.handleElementEnded=()=>{if(!this.playIntent||this.endedReported)return;let e=this.loadedUrl;e&&(this.endedReported=!0,this.emit(`trackEnded`,e))},this.pauseRetryDelayMs=t.pauseRetryDelayMs??pt,this.audio=e,this.audio.crossOrigin=`anonymous`,this.audio.addEventListener(`play`,this.handleElementPlay),this.audio.addEventListener(`pause`,this.handleElementPause),this.audio.addEventListener(`timeupdate`,this.handleElementTimeUpdate),this.audio.addEventListener(`durationchange`,this.handleElementDurationChange),this.audio.addEventListener(`loadstart`,this.handleElementDataWait),this.audio.addEventListener(`waiting`,this.handleElementDataWait),this.audio.addEventListener(`stalled`,this.handleElementDataWait),this.audio.addEventListener(`canplay`,this.handleElementDataReady),this.audio.addEventListener(`playing`,this.handleElementPlaying),this.audio.addEventListener(`emptied`,this.handleElementDataReady),this.audio.addEventListener(`error`,this.handleElementError),this.audio.addEventListener(`ended`,this.handleElementEnded)}get isPlaying(){return this.playing}get hasTrack(){return this.loadedUrl!==void 0}get isBuffering(){return this.buffering}get lastError(){return this.error}get currentTimeSecs(){return this.audio.currentTime}get durationSecs(){let e=this.audio.duration;return Number.isFinite(e)?e:0}get volume(){return this.audio.volume}set volume(e){this.audio.volume=Math.min(1,Math.max(0,e))}load(e){if(this.loadedUrl===e.url){this.playIntent&&this.attemptPlay();return}this.loadedUrl=e.url,this.endedReported=!1,this.pauseRetryUsed=!1,this.setError(void 0),this.audio.src=e.url,this.playIntent&&this.attemptPlay()}play(){this.playIntent=!0,this.pauseRequested=!1,this.pauseRetryUsed=!1,this.loadedUrl&&this.attemptPlay()}playWhenLoaded(){this.clearPauseRetry(),this.pauseRequested=!0,this.audio.removeAttribute(`src`),this.audio.load(),this.loadedUrl=void 0,this.endedReported=!1,this.setPlaying(!1),this.setError(void 0),this.playIntent=!0,this.pauseRetryUsed=!1}pause(){this.playIntent=!1,this.pauseRequested=!0,this.clearPauseRetry(),this.audio.pause(),this.setBuffering(!1)}seekTo(e){let t=Math.max(0,e);if(this.audio.readyState<HTMLMediaElement.HAVE_METADATA){this.audio.currentTime=t;return}let n=this.durationSecs;this.audio.currentTime=n>0?Math.min(n,t):t}cancelPlayIntent(){this.playIntent=!1}on(e,t){this.listeners[e].add(t)}off(e,t){this.listeners[e].delete(t)}dispose(){this.playIntent=!1,this.pauseRequested=!0,this.clearPauseRetry(),this.audio.pause(),this.setPlaying(!1),this.setBuffering(!1),this.audio.removeEventListener(`play`,this.handleElementPlay),this.audio.removeEventListener(`pause`,this.handleElementPause),this.audio.removeEventListener(`timeupdate`,this.handleElementTimeUpdate),this.audio.removeEventListener(`durationchange`,this.handleElementDurationChange),this.audio.removeEventListener(`loadstart`,this.handleElementDataWait),this.audio.removeEventListener(`waiting`,this.handleElementDataWait),this.audio.removeEventListener(`stalled`,this.handleElementDataWait),this.audio.removeEventListener(`canplay`,this.handleElementDataReady),this.audio.removeEventListener(`playing`,this.handleElementPlaying),this.audio.removeEventListener(`emptied`,this.handleElementDataReady),this.audio.removeEventListener(`error`,this.handleElementError),this.audio.removeEventListener(`ended`,this.handleElementEnded),this.audio.removeAttribute(`src`),this.audio.load(),this.loadedUrl=void 0,this.listeners.playingChanged.clear(),this.listeners.timeChanged.clear(),this.listeners.durationChanged.clear(),this.listeners.bufferingChanged.clear(),this.listeners.errorChanged.clear(),this.listeners.playbackStarted.clear(),this.listeners.trackEnded.clear()}attemptPlay(){this.audio.error&&this.loadedUrl&&(this.audio.src=this.loadedUrl),this.audio.play().catch(e=>{let t=ft(e);if(t!==`benign`){if(this.setBuffering(!1),t===`blocked`){this.playIntent=!1;return}this.error?.url===this.loadedUrl&&this.error?.mediaErrorCode!==void 0||this.setError({url:this.loadedUrl})}})}setPlaying(e){this.playing!==e&&(this.playing=e,this.emit(`playingChanged`,e))}setBuffering(e){this.buffering!==e&&(this.buffering=e,this.emit(`bufferingChanged`,e))}setError(e){mt(this.error,e)||(this.error=e,this.emit(`errorChanged`,e))}emit(e,t){for(let n of this.listeners[e])n(t)}clearPauseRetry(){this.pauseRetryTimer!==void 0&&(window.clearTimeout(this.pauseRetryTimer),this.pauseRetryTimer=void 0)}};function gt(e=new Audio){let t=.5;try{return e.volume=t,e.volume===t}catch{return!1}}function _t(e,t){return e<100?-1:+(e>t-100)}function vt(e,t,n){return e<t+n/2}function yt(e,t,n,r){let i=e.indexOf(t);if(i===-1)return e;let a=[...e];a.splice(i,1);let o=a.indexOf(n);return o===-1?e:(a.splice(r?o:o+1,0,t),a)}function bt(e,t){let n=[...new Set(e)].filter(e=>t.includes(e)),r=t.filter(e=>!n.includes(e));return[...n,...r]}var xt={show:8,artist:12},U=class extends c{constructor(...e){super(...e),this.heading=``,this.loading=!1,this.placeholderVariant=`show`,this.mode=`desktop`,this.capabilities=j}render(){return a`
      <section class=${this.mode} aria-busy=${this.loading?`true`:`false`}>
        <div class="heading-row">
          ${this.gripTemplate}
          <h2>${this.headingContentTemplate}</h2>
          ${this.showMoreTemplate}
        </div>
        ${this.loading?this.placeholderTemplate:this.loadedBodyTemplate}
      </section>
    `}get gripTemplate(){return this.sectionId?a`
      <div
        class="reorder-grip"
        title=${o(`Drag to reorder`)}
        @mousedown=${this.handleGripMouseDown}
        @mouseup=${this.handleGripMouseUp}
      >
        ${k(`grip-vertical`,{label:o(`Drag to reorder`)})}
      </div>
    `:n}get showMoreAvailable(){let e=this.sectionId;return e!==void 0&&this.capabilities.showMoreSections.includes(e)}get headingContentTemplate(){return this.showMoreAvailable?a`
      <button class="heading-button" @click=${this.handleShowMoreClick}>
        ${this.heading}
      </button>
    `:this.heading}get showMoreTemplate(){return this.showMoreAvailable?a`
      <button class="show-more" @click=${this.handleShowMoreClick}>
        ${o(`Show more`)}
      </button>
    `:n}handleShowMoreClick(e){let t=this.sectionId;if(!t)return;let n=e.currentTarget;n instanceof HTMLElement&&n.focus(),this.dispatchEvent(N(A.showMoreRequested,{section:t}))}handleGripMouseDown(e){if(e.button!==0)return;let t=this.sectionId;t&&this.dispatchEvent(N(A.reorderGripPressed,{section:t}))}handleGripMouseUp(e){if(e.button!==0)return;let t=this.sectionId;t&&this.dispatchEvent(N(A.reorderGripReleased,{section:t}))}get placeholderTemplate(){let e=this.placeholderVariant===`artist`?`artist`:`show`,t=e===`artist`?()=>this.artistPlaceholderTileTemplate:()=>this.showPlaceholderTileTemplate;return a`
      <div class="placeholder-row ${e}" aria-hidden="true">
        ${ee(m(xt[e]),t)}
      </div>
    `}get showPlaceholderTileTemplate(){return a`
      <div class="placeholder-tile">
        <lma-tombstone class="placeholder-art"></lma-tombstone>
        <lma-tombstone
          class="placeholder-line placeholder-title"
        ></lma-tombstone>
        <lma-tombstone
          class="placeholder-line placeholder-meta"
        ></lma-tombstone>
      </div>
    `}get artistPlaceholderTileTemplate(){return a`
      <div class="placeholder-tile">
        <lma-tombstone class="placeholder-circle"></lma-tombstone>
        <lma-tombstone
          class="placeholder-line placeholder-name"
        ></lma-tombstone>
        <lma-tombstone
          class="placeholder-line placeholder-count"
        ></lma-tombstone>
      </div>
    `}get loadedBodyTemplate(){return a`
      <slot><p class="section-note">${o(`Coming soon`)}</p></slot>
    `}static get styles(){return e`
      :host {
        display: block;
      }

      .heading-row {
        display: flex;
        align-items: center;
        gap: 8px;
        margin: 0 0 12px;
      }

      h2 {
        margin: 0;
        font-size: 18px;
        line-height: 28px;
        font-weight: 600;
      }

      /* A native button styled as the plain heading text. */
      .heading-button {
        padding: 0;
        border: 0;
        background: none;
        color: inherit;
        font: inherit;
        cursor: pointer;
      }

      .heading-button:focus-visible {
        outline: 2px solid var(--ia-theme-link-color, #4b64ff);
        outline-offset: 2px;
      }

      /*
       * The grip's padding widens its mouse target past the 16px icon
       * without changing the heading row's height (the h2 line box is
       * taller than the padded grip).
       */
      .reorder-grip {
        display: flex;
        align-items: center;
        padding: 2px;
        color: var(--ia-theme-secondary-text-color, #666);
        cursor: grab;
      }

      .reorder-grip:hover {
        color: var(--ia-theme-link-color, #4b64ff);
      }

      .reorder-grip:active {
        cursor: grabbing;
      }

      /* A text-styled native button; the extra left margin widens the gap
         after the heading to 16px. */
      .show-more {
        margin-left: 8px;
        padding: 0;
        border: 0;
        background: none;
        color: var(--ia-theme-link-color, #4b64ff);
        font-family: inherit;
        font-size: 12px;
        line-height: 16px;
        cursor: pointer;
        transition: color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
      }

      .show-more:hover {
        color: var(--ia-theme-primary-cta-fill, #194880);
      }

      /*
       * The row mirrors the show-tile rail's gap and padding and scrolls
       * the same way, with the rail's transparent-until-hover scrollbar,
       * so both reserve the same scrollbar gutter and the loaded rail
       * lands where the placeholder row sat. Modern engines honor
       * scrollbar-width/scrollbar-color; the -webkit- rules cover older
       * Safari and Chromium.
       */
      .placeholder-row {
        display: flex;
        gap: 24px;
        overflow-x: auto;
        padding: 8px 4px;
        scrollbar-width: thin;
        scrollbar-color: transparent transparent;
      }

      .placeholder-row:hover,
      .placeholder-row:focus-within {
        scrollbar-color: var(--ia-theme-primary-cta-border, #c5d1df) transparent;
      }

      .placeholder-row::-webkit-scrollbar {
        height: 8px;
      }

      .placeholder-row::-webkit-scrollbar-track {
        background: transparent;
      }

      .placeholder-row::-webkit-scrollbar-thumb {
        border-radius: 4px;
        background: transparent;
      }

      .placeholder-row:hover::-webkit-scrollbar-thumb,
      .placeholder-row:focus-within::-webkit-scrollbar-thumb {
        background: var(--ia-theme-primary-cta-border, #c5d1df);
      }

      /*
       * Placeholder pieces are lma-tombstone elements, which own the
       * shared gradient, shimmer, and default corner radius; the frame
       * composes and sizes them to each variant's loaded footprint.
       */
      .placeholder-tile {
        flex-shrink: 0;
      }

      /*
       * Each show placeholder spans the footprint a one-line-title show
       * tile occupies in its rail: a 220px art square plus 92px of text
       * rows and margins (12 art margin + 20 title + 4 title margin +
       * 16 artist + 16 date + 8 date margin + 16 rating row). Together
       * with the row reserving the rail's scrollbar gutter, the
       * loading-to-loaded swap does not reflow the page. Within that
       * footprint the placeholder mirrors the tile's anatomy: the art
       * square with the loaded art's corner radius, then two shorter bars
       * for the title and meta text lines.
       */
      .placeholder-row.show .placeholder-tile {
        width: 220px;
        height: 312px;
      }

      /* Show tiles are 28vw wide in the mobile rail. */
      section.mobile .placeholder-row.show .placeholder-tile {
        width: 28vw;
        height: calc(28vw + 92px);
      }

      .placeholder-art {
        width: 100%;
        aspect-ratio: 1;
        height: auto;
        margin-bottom: 12px;
        border-radius: 8px;
      }

      .placeholder-line {
        height: 16px;
      }

      .placeholder-title {
        width: 85%;
        height: 20px;
        margin-bottom: 8px;
      }

      .placeholder-meta {
        width: 60%;
      }

      /*
       * Each artist placeholder mirrors a rendered artist tile's centered
       * circle-name-count anatomy: a 144px circle over a name bar and a
       * show-count bar. The column is laid out with flex so the bars center
       * under the circle and the fixed margins sum without collapsing. The
       * total footprint (8 top + 144 circle + 8 + 16 name + 8 + 12 count =
       * 196px) matches a loaded artist tile, so the swap does not reflow.
       * Artist tiles keep the same width in every display mode, so there is
       * no mobile override.
       */
      .placeholder-row.artist .placeholder-tile {
        display: flex;
        flex-direction: column;
        align-items: center;
        width: 144px;
      }

      .placeholder-circle {
        flex-shrink: 0;
        width: 144px;
        height: 144px;
        margin: 8px 0;
        border-radius: 50%;
      }

      .placeholder-name {
        width: 60%;
        height: 16px;
        margin-bottom: 8px;
      }

      .placeholder-count {
        width: 40%;
        height: 12px;
      }

      .section-note {
        margin: 0;
        font-size: var(--ia-theme-font-size-standard, 1.4rem);
        color: var(--ia-theme-secondary-text-color, #666);
      }
    `}};y([t({type:String})],U.prototype,`sectionId`,void 0),y([t({type:String})],U.prototype,`heading`,void 0),y([t({type:Boolean})],U.prototype,`loading`,void 0),y([t({type:String})],U.prototype,`placeholderVariant`,void 0),y([t({type:String})],U.prototype,`mode`,void 0),y([t({type:Object})],U.prototype,`capabilities`,void 0),y([t({type:Object})],U.prototype,`searchAdapter`,void 0),y([t({type:Object})],U.prototype,`userState`,void 0),y([t({type:Object})],U.prototype,`artistCounts`,void 0),y([t({type:Object})],U.prototype,`showDetails`,void 0),U=y([h(`lma-section-frame`),i()],U);var St=24,W=class extends c{constructor(...e){super(...e),this.heading=``,this.servicesLoading=!1,this.mode=`desktop`,this.capabilities=j,this.query=``,this.artists=[],this.fetchState=`loading`,this.countsPending=!1,this.requestGeneration=0}willUpdate(e){if(e.has(`searchAdapter`)||e.has(`query`)){this.fetchArtists();return}e.has(`artistCounts`)&&this.artists.length>0&&this.mergeShowCounts(this.requestGeneration)}render(){let e=this.bodyLoading;return a`
      <lma-section-frame
        sectionId="artists"
        heading=${this.heading}
        ?loading=${e}
        mode=${this.mode}
        placeholderVariant="artist"
        .capabilities=${this.capabilities}
        @reorderGripPressed=${this.handleReorderGripPressed}
        @reorderGripReleased=${this.handleReorderGripReleased}
        @showMoreRequested=${this.handleShowMoreRequested}
      >
        ${e?n:this.bodyTemplate}
      </lma-section-frame>
      <div class="visually-hidden" role="status" aria-live="polite">
        ${this.statusText}
      </div>
    `}get bodyLoading(){return this.servicesLoading||this.searchAdapter!==void 0&&this.fetchState===`loading`}get fetchFailed(){return this.searchAdapter===void 0||this.fetchState===`error`}get statusText(){if(this.bodyLoading)return``;if(this.fetchFailed)return o(`Could not load artists.`);let e=this.artists.length;return e===0?o(`No artists found.`):S(e,{one:()=>o(`1 artist`),other:e=>o(r`${e} artists`)})}get bodyTemplate(){return this.fetchFailed?a`
        <p class="rail-note error">${o(`Could not load artists.`)}</p>
      `:this.artists.length===0?a`<p class="rail-note">${o(`No artists found.`)}</p>`:this.railTemplate}get railTemplate(){return a`
      <ul class="rail" role="list">
        ${this.artists.map(e=>a`
            <li>
              <lma-artist-tile
                previewable
                .artist=${e}
                mode=${this.mode}
                ?countsPending=${this.countsPending}
                @artistSelected=${this.handleArtistSelected}
                @artistHovered=${this.handleArtistHovered}
                @artistHoverEnded=${this.handleArtistHoverEnded}
              ></lma-artist-tile>
            </li>
          `)}
      </ul>
    `}async fetchArtists(){let e=this.searchAdapter;if(!e)return;let t=++this.requestGeneration;this.fetchState=`loading`,this.artistCounts?.primeCounts();try{let n=await e.getArtistCollections({rows:St,query:this.query});if(t!==this.requestGeneration)return;this.artists=n,this.fetchState=`loaded`}catch{if(t!==this.requestGeneration)return;this.fetchState=`error`,this.countsPending=!1;return}await this.mergeShowCounts(t)}async mergeShowCounts(e){let t=this.artistCounts;if(!t){this.countsPending=!1;return}this.countsPending=!0;let n=await t.get(this.artists.map(e=>e.identifier));e===this.requestGeneration&&(this.artists=this.artists.map(e=>({...e,showCount:n.get(e.identifier)??null})),this.countsPending=!1)}handleArtistSelected(e){this.dispatchEvent(N(A.artistSelected,e.detail))}handleArtistHovered(e){this.dispatchEvent(N(A.artistHovered,e.detail))}handleArtistHoverEnded(e){this.dispatchEvent(N(A.artistHoverEnded,e.detail))}handleReorderGripPressed(e){this.dispatchEvent(N(A.reorderGripPressed,e.detail))}handleReorderGripReleased(e){this.dispatchEvent(N(A.reorderGripReleased,e.detail))}handleShowMoreRequested(e){this.dispatchEvent(N(A.showMoreRequested,e.detail))}static get styles(){return e`
      :host {
        display: block;
      }

      /*
       * The rail's scrollbar occupies a constant strip whose thumb is
       * transparent until the rail is hovered or holds keyboard focus, so
       * showing it never shifts the layout and the rail stays scrollable
       * at all times. Modern engines honor
       * scrollbar-width/scrollbar-color; the -webkit- rules cover older
       * Safari and Chromium.
       */
      .rail {
        display: flex;
        flex-wrap: nowrap;
        gap: 24px;
        overflow-x: auto;
        margin: 0;
        padding: 8px 4px;
        list-style: none;
        scrollbar-width: thin;
        scrollbar-color: transparent transparent;
      }

      .rail:hover,
      .rail:focus-within {
        scrollbar-color: var(--ia-theme-primary-cta-border, #c5d1df) transparent;
      }

      .rail::-webkit-scrollbar {
        height: 8px;
      }

      .rail::-webkit-scrollbar-track {
        background: transparent;
      }

      .rail::-webkit-scrollbar-thumb {
        border-radius: 4px;
        background: transparent;
      }

      .rail:hover::-webkit-scrollbar-thumb,
      .rail:focus-within::-webkit-scrollbar-thumb {
        background: var(--ia-theme-primary-cta-border, #c5d1df);
      }

      /* Artist tiles keep the same width in every display mode. */
      .rail li {
        flex-shrink: 0;
        width: 144px;
      }

      /* The list item is the rail's flex item and the tile fills it, the
         same height chain as the show rails; artist tiles are fixed-height,
         so the stretch leaves every row uniform. */
      .rail lma-artist-tile {
        height: 100%;
      }

      .rail-note {
        margin: 0;
        font-size: var(--ia-theme-font-size-standard, 1.4rem);
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .rail-note.error {
        color: var(--ia-theme-color-danger, #e51c23);
      }

      /* http://webaim.org/techniques/css/invisiblecontent/ */
      .visually-hidden {
        position: absolute;
        clip: rect(0, 0, 0, 0);
        width: 1px;
        height: 1px;
        overflow: hidden;
      }
    `}};y([t({type:String})],W.prototype,`heading`,void 0),y([t({type:Boolean})],W.prototype,`servicesLoading`,void 0),y([t({type:String})],W.prototype,`mode`,void 0),y([t({type:Object})],W.prototype,`capabilities`,void 0),y([t({type:Object})],W.prototype,`searchAdapter`,void 0),y([t({type:Object})],W.prototype,`artistCounts`,void 0),y([t({type:String})],W.prototype,`query`,void 0),y([s()],W.prototype,`artists`,void 0),y([s()],W.prototype,`fetchState`,void 0),y([s()],W.prototype,`countsPending`,void 0),W=y([h(`lma-artists-section`),i()],W);function Ct(e,t){let{maxPerArtist:n,limit:r}=t,i=new Map,a=[];for(let t of e){if(a.length>=r)break;let e=i.get(t.artist)??0;e>=n||(i.set(t.artist,e+1),a.push(t))}return a}function wt(e,t){let n=t.trim().toLowerCase();return n?[e.title,e.artist,e.venue??``,T(e.date)].some(e=>e.toLowerCase().includes(n)):!0}var G=24,Tt=80,Et=2,Dt=24,Ot=24,kt=12,K={shows:{queryDriven:!0,async fetchShows(e,{query:t}){return t.trim()?e.searchShows({query:t,rows:G}):Ct(await e.topShows({rows:Tt}),{maxPerArtist:Et,limit:G})}},favorites:{queryDriven:!1,accountScoped:!0,async fetchShows(e,{favoritesCollection:t}){return t?e.favoriteShows(t,{rows:Dt}):[]}},highlyRated:{queryDriven:!1,async fetchShows(e){return e.topRatedShows({rows:Ot})}},freshlyUploaded:{queryDriven:!1,async fetchShows(e){return e.freshlyUploaded({rows:kt})}},onThisDay:{queryDriven:!1,async fetchShows(e){let t=new Date;return e.onThisDay({month:t.getMonth()+1,day:t.getDate()})}}};function At(e){return e in K}var q=class extends c{constructor(...e){super(...e),this.sectionId=`shows`,this.heading=``,this.servicesLoading=!1,this.mode=`desktop`,this.capabilities=j,this.query=``,this.favoritedShows=[],this.unfavoritedShowIds=new Set,this.shows=[],this.fetchState=`loading`,this.requestGeneration=0}willUpdate(e){let t=this.railConfig;if(t){let n=e.has(`query`)&&t.queryDriven,r=e.has(`favoritesCollection`)&&t.accountScoped===!0;(e.has(`searchAdapter`)||e.has(`sectionId`)||n||r)&&this.fetchShows()}this.hidden=this.hiddenForQuery}get railConfig(){if(At(this.sectionId))return K[this.sectionId]}render(){if(this.hiddenForQuery)return a``;let e=this.bodyLoading;return a`
      <lma-section-frame
        sectionId=${this.sectionId}
        heading=${this.heading}
        ?loading=${e}
        mode=${this.mode}
        .capabilities=${this.capabilities}
        @reorderGripPressed=${this.handleReorderGripPressed}
        @reorderGripReleased=${this.handleReorderGripReleased}
        @showMoreRequested=${this.handleShowMoreRequested}
      >
        ${e?n:this.bodyTemplate}
      </lma-section-frame>
      <div class="visually-hidden" role="status" aria-live="polite">
        ${this.statusText}
      </div>
    `}get accountPending(){return this.railConfig?.accountScoped===!0&&this.favoritesCollection===void 0}get accountUnavailable(){return this.railConfig?.accountScoped===!0&&this.favoritesCollection===P}get bodyLoading(){return this.servicesLoading?!0:this.accountUnavailable?!1:this.searchAdapter!==void 0&&(this.accountPending||this.fetchState===`loading`)}get fetchFailed(){return this.searchAdapter===void 0||this.accountUnavailable||this.fetchState===`error`}get displayShows(){let e=this.railConfig,t=e?.accountScoped?this.showsWithSessionFavorites:this.shows;return!e||e.queryDriven?t:t.filter(e=>wt(e,this.query))}get showsWithSessionFavorites(){let e=this.favoritedShows,t=this.unfavoritedShowIds;if(e.length===0&&t.size===0)return this.shows;let n=new Set(e.map(e=>e.identifier)),r=this.shows.filter(e=>!n.has(e.identifier));return[...e,...r].filter(e=>!t.has(e.identifier))}get hiddenForQuery(){return this.bodyLoading||this.fetchFailed||!this.query.trim()?!1:this.displayShows.length===0}get statusText(){if(this.bodyLoading)return``;if(this.fetchFailed)return o(`Could not load shows.`);let e=this.displayShows.length;return e===0?this.emptyText:S(e,{one:()=>o(`1 show`),other:e=>o(r`${e} shows`)})}get bodyTemplate(){return this.fetchFailed?a`
        <p class="rail-note error">${o(`Could not load shows.`)}</p>
      `:this.displayShows.length===0?a`<p class="rail-note">${this.emptyText}</p>`:this.railTemplate}get emptyText(){return this.railConfig?.accountScoped?o(`No favorite shows yet. Use the heart on a show in your queue, or the Favorite button on a show's page, to add one.`):o(`No shows found.`)}get railTemplate(){return a`
      <ul class="rail ${this.mode}" role="list">
        ${this.displayShows.map(e=>a`
            <li>
              <lma-show-tile
                previewable
                .show=${e}
                mode=${this.mode}
                .capabilities=${this.capabilities}
                @showSelected=${this.handleShowSelected}
                @showPlayRequested=${this.handleShowPlayRequested}
                @showQueueRequested=${this.handleShowQueueRequested}
                @showPreviewRequested=${this.handleShowPreviewRequested}
                @showPreviewResumed=${this.handleShowPreviewResumed}
                @showPreviewEnded=${this.handleShowPreviewEnded}
              ></lma-show-tile>
            </li>
          `)}
      </ul>
    `}async fetchShows(){let e=this.searchAdapter,t=this.railConfig;if(!e||!t)return;let n=++this.requestGeneration;this.fetchState=`loading`;let r=typeof this.favoritesCollection==`string`?this.favoritesCollection:void 0;if(!(t.accountScoped&&!r))try{let i=await t.fetchShows(e,{query:this.query,favoritesCollection:r});if(n!==this.requestGeneration)return;this.shows=i,this.fetchState=`loaded`}catch{if(n!==this.requestGeneration)return;this.fetchState=`error`}}handleShowSelected(e){this.dispatchEvent(N(A.showSelected,e.detail))}handleShowPlayRequested(e){this.dispatchEvent(N(A.showPlayRequested,e.detail))}handleShowQueueRequested(e){this.dispatchEvent(N(A.showQueueRequested,e.detail))}handleShowPreviewRequested(e){this.dispatchEvent(N(A.showPreviewRequested,e.detail))}handleShowPreviewResumed(e){this.dispatchEvent(N(A.showPreviewResumed,e.detail))}handleShowPreviewEnded(e){this.dispatchEvent(N(A.showPreviewEnded,e.detail))}handleReorderGripPressed(e){this.dispatchEvent(N(A.reorderGripPressed,e.detail))}handleReorderGripReleased(e){this.dispatchEvent(N(A.reorderGripReleased,e.detail))}handleShowMoreRequested(e){this.dispatchEvent(N(A.showMoreRequested,e.detail))}static get styles(){return e`
      :host {
        display: block;
      }

      /*
       * The host carries the hidden attribute while a query leaves the
       * rail with nothing to display (see hiddenForQuery). The explicit
       * rule is required: the :host display above outweighs the
       * user-agent [hidden] rule.
       */
      :host([hidden]) {
        display: none;
      }

      /*
       * The rail's scrollbar occupies a constant strip whose thumb is
       * transparent until the rail is hovered or holds keyboard focus, so
       * showing it never shifts the layout and the rail stays scrollable
       * at all times. Modern engines honor
       * scrollbar-width/scrollbar-color; the -webkit- rules cover older
       * Safari and Chromium.
       */
      .rail {
        display: flex;
        flex-wrap: nowrap;
        gap: 24px;
        overflow-x: auto;
        margin: 0;
        padding: 8px 4px;
        list-style: none;
        scrollbar-width: thin;
        scrollbar-color: transparent transparent;
      }

      .rail:hover,
      .rail:focus-within {
        scrollbar-color: var(--ia-theme-primary-cta-border, #c5d1df) transparent;
      }

      .rail::-webkit-scrollbar {
        height: 8px;
      }

      .rail::-webkit-scrollbar-track {
        background: transparent;
      }

      .rail::-webkit-scrollbar-thumb {
        border-radius: 4px;
        background: transparent;
      }

      .rail:hover::-webkit-scrollbar-thumb,
      .rail:focus-within::-webkit-scrollbar-thumb {
        background: var(--ia-theme-primary-cta-border, #c5d1df);
      }

      .rail li {
        flex-shrink: 0;
        width: 220px;
      }

      .rail.mobile li {
        width: 28vw;
      }

      /* The list item is the rail's flex item, stretched to the row's
         tallest tile; the tile fills it so every tile in the row shares
         that height and bottom-aligns its rating row. */
      .rail lma-show-tile {
        height: 100%;
      }

      .rail-note {
        margin: 0;
        font-size: var(--ia-theme-font-size-standard, 1.4rem);
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .rail-note.error {
        color: var(--ia-theme-color-danger, #e51c23);
      }

      /* http://webaim.org/techniques/css/invisiblecontent/ */
      .visually-hidden {
        position: absolute;
        clip: rect(0, 0, 0, 0);
        width: 1px;
        height: 1px;
        overflow: hidden;
      }
    `}};y([t({type:String})],q.prototype,`sectionId`,void 0),y([t({type:String})],q.prototype,`heading`,void 0),y([t({type:Boolean})],q.prototype,`servicesLoading`,void 0),y([t({type:String})],q.prototype,`mode`,void 0),y([t({type:Object})],q.prototype,`capabilities`,void 0),y([t({type:Object})],q.prototype,`searchAdapter`,void 0),y([t({type:Object})],q.prototype,`userState`,void 0),y([t({type:String})],q.prototype,`query`,void 0),y([t({attribute:!1})],q.prototype,`favoritesCollection`,void 0),y([t({type:Array})],q.prototype,`favoritedShows`,void 0),y([t({type:Object})],q.prototype,`unfavoritedShowIds`,void 0),y([s()],q.prototype,`shows`,void 0),y([s()],q.prototype,`fetchState`,void 0),q=y([h(`lma-show-rail-section`),i()],q);var jt=12;function J(e){return e+12+jt}J(500);function Mt(e,t,n,r=500){let i=Math.max(0,Math.min(r,t.width-32)),a=Math.max(0,t.height-16),o=Math.min(Math.max(0,n),a),s=t.width-e.right>=J(r)?`right`:`left`,c=s===`right`?e.right+12:e.left-i-12,l=Math.max(16,t.width-i-16),u=Math.min(Math.max(c,16),l),d=t.height-o-16;return{side:s,top:Math.max(16,Math.min(e.top,d)),left:u,width:i,maxHeight:a}}var Y=class extends c{focusableControls(){return[...this.shadowRoot?.querySelectorAll(`a[href], button:not([disabled])`)??[]]}focusFirstControl(){this.focusableControls()[0]?.focus()}render(){let e=this.artist;if(!e)return a``;let t=o(`Close`);return a`
      <div class="scroll">
        <div class="banner">
          <img src=${e.imageUrl} alt="" decoding="async" />
          <div class="banner-shade" aria-hidden="true"></div>
        </div>
        <div class="body">
          <div class="name">${e.name}</div>
          ${this.showCountTemplate(e)} ${this.aboutTemplate}
        </div>
      </div>
      <button
        class="close"
        title=${t}
        aria-label=${t}
        @click=${this.handleCloseClick}
      >
        ${k(`x`,{size:14})}
      </button>
    `}showCountTemplate(e){let t=e.showCount;return t===null?n:a`<div class="show-count">${S(t,{one:()=>o(`1 show`),other:e=>o(r`${e} shows`)})}</div>`}get aboutTemplate(){let e=this.aboutState;return e?.status===`loaded`?a`
      <div class="about">
        <h3 class="section-label">${o(`About`)}</h3>
        <div class="about-text">${d(e.extractHTML)}</div>
        <a
          class="wikipedia-link"
          href=${e.articleUrl}
          target="_blank"
          rel="noopener"
          >${o(`From Wikipedia`)}</a
        >
      </div>
    `:n}handleCloseClick(){this.dispatchEvent(N(A.closeRequested,null))}static get styles(){return e`
      :host {
        display: flex;
        flex-direction: column;
        position: relative;
        box-sizing: border-box;
        overflow: hidden;
        background: var(--ia-theme-secondary-background-color, #fff);
        color: var(--ia-theme-primary-text-color, #2c2c2c);
        font-family: var(--ia-theme-base-font-family, inherit);
        border-radius: 12px;
        box-shadow:
          0 25px 60px rgba(0, 0, 0, 0.18),
          0 0 0 1px rgba(0, 0, 0, 0.07);
      }

      .scroll {
        flex: 1 1 auto;
        min-height: 0;
        overflow-y: auto;
      }

      .banner {
        position: relative;
        height: 180px;
        overflow: hidden;
        /* The banner backdrop is fixed near-black in every theme: artist
           images are letterboxed onto a dark plate, which no theme token
           describes. */
        background: #171717;
      }

      .banner img {
        display: block;
        width: 100%;
        height: 100%;
        object-fit: contain;
      }

      .banner-shade {
        position: absolute;
        inset: 0;
        pointer-events: none;
        background: linear-gradient(
          to top,
          rgba(0, 0, 0, 0.5),
          transparent 50%,
          transparent
        );
      }

      .body {
        padding: 16px;
      }

      .name {
        font-size: 16px;
        line-height: 1.25;
        font-weight: 700;
        color: var(--ia-theme-primary-text-color, #2c2c2c);
      }

      .show-count {
        margin-top: 2px;
        font-size: 12px;
        line-height: 16px;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .about {
        margin-top: 12px;
        padding-top: 12px;
        border-top: 1px solid var(--ia-theme-primary-cta-border, #c5d1df);
      }

      .section-label {
        margin: 0 0 4px;
        font-size: 12px;
        line-height: 16px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.025em;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .about-text {
        font-size: 12px;
        line-height: 1.625;
        color: var(--ia-theme-primary-text-color, #2c2c2c);
      }

      .about-text p {
        margin: 0;
      }

      .wikipedia-link {
        display: inline-block;
        margin-top: 4px;
        font-size: 10px;
        line-height: 14px;
        color: var(--ia-theme-link-color, #4b64ff);
        text-decoration: none;
      }

      .wikipedia-link:hover {
        text-decoration: underline;
      }

      .close {
        position: absolute;
        top: 12px;
        right: 12px;
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        border: 0;
        border-radius: 9999px;
        background: var(--ia-theme-primary-background-color, #fbfbfd);
        color: var(--ia-theme-secondary-text-color, #666);
        cursor: pointer;
        transition: background-color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
      }

      .close:hover {
        background: var(--ia-theme-primary-cta-border, #c5d1df);
      }
    `}};y([t({type:Object})],Y.prototype,`artist`,void 0),y([t({attribute:!1})],Y.prototype,`aboutState`,void 0),Y=y([h(`lma-artist-info-card`),i()],Y);function X(e){return e.button===0&&!e.metaKey&&!e.ctrlKey&&!e.shiftKey&&!e.altKey}function Nt(e){let t=ge(e);if(t)return{identifier:t,name:e.artist,showCount:null,imageUrl:_e(e)??``}}var Pt=4,Z=class extends c{constructor(...e){super(...e),this.capabilities=j}focusableControls(){return[...this.shadowRoot?.querySelectorAll(`a[href], button:not([disabled])`)??[]]}focusFirstControl(){this.focusableControls()[0]?.focus()}render(){let e=this.show;if(!e)return a``;let t=o(`Close`);return a`
      <div class="scroll">
        <div class="header">
          <div class="headlines">
            <a
              class="headline date"
              href=${I(e.identifier)}
              @click=${this.handleDateClick}
              >${T(e.date)}</a
            >
            <div class="headline artist">${e.artist}</div>
            ${e.venue?a`<div class="headline venue">${e.venue}</div>`:n}
            <div class="by">${this.byArtistTemplate(e)}</div>
          </div>
          <div class="rating">${this.ratingTemplate(e)}</div>
        </div>
        ${this.provenanceTemplate(e)} ${this.reviewsRowTemplate(e)}
        <div class="setlist-header">
          <h3 class="setlist-heading">${o(`Setlist`)}</h3>
          ${this.setlistActionsTemplate}
        </div>
        ${this.setlistTemplate(e)}
      </div>
      <button
        class="close"
        title=${t}
        aria-label=${t}
        @click=${this.handleCloseClick}
      >
        ${k(`x`,{size:14})}
      </button>
    `}get setlistActionsTemplate(){return this.capabilities.playbackAvailable?a`
      <div class="setlist-actions">
        <button class="queue-all" @click=${this.handleQueueAllClick}>
          ${o(`Queue All`)}
        </button>
        <button class="play-all" @click=${this.handlePlayAllClick}>
          ${k(`play`,{size:12})} ${o(`Play All`)}
        </button>
      </div>
    `:n}trackActionsTemplate(e,t){if(!this.capabilities.playbackAvailable)return n;let i=o(r`Add ${e.title} to queue`),s=o(r`Play ${e.title}`);return a`
      <button
        class="track-action track-queue"
        title=${i}
        aria-label=${i}
        @click=${()=>this.requestQueue(t)}
      >
        ${k(`plus`,{size:12})}
      </button>
      <button
        class="track-action track-play"
        title=${s}
        aria-label=${s}
        @click=${()=>this.requestPlay(t)}
      >
        ${k(`play`,{size:12})}
      </button>
    `}byArtistTemplate(e){let t=Nt(e);return o(t?a`by ${a`<a
      class="artist-link"
      href=${L(t.identifier)}
      @click=${e=>this.handleArtistClick(e,t)}
      >${t.name}</a
    >`}`:a`by ${e.artist}`)}ratingTemplate(e){let t=e.avgRating;if(t===null)return a`<span class="not-rated">${o(`Not yet rated`)}</span>`;let n=Math.round(t),i=t.toLocaleString(void 0,{minimumFractionDigits:1,maximumFractionDigits:1}),s=[1,2,3,4,5].map(e=>a`<span class="star ${e<=n?`filled`:``}"
        >${k(`star`,{size:14})}</span
      >`);return a`
      <span
        class="stars"
        role="img"
        aria-label=${o(r`Rated ${i} out of 5`)}
        >${s}</span
      >
      <span class="rating-label">${o(`Rating`)}</span>
    `}provenanceTemplate(e){let t=this.setlistState?.status===`loaded`?this.setlistState:void 0,r=e.source||t?.source,i=e.lineage||t?.lineage;return!r&&!i?n:a`
      <div class="provenance">${r?a`<div class="section-label">${o(`Source`)}</div>
          <div class="provenance-text source" title=${r}>${r}</div>`:n} ${i?a`<div class="section-label">${o(`Lineage`)}</div>
          <div class="provenance-text lineage" title=${i}>
            ${i}
          </div>`:n}</div>
    `}reviewsRowTemplate(e){let t=e.numReviews;if(t<1)return n;let i=o(t===1?`1 Review`:r`${t.toLocaleString()} Reviews`);return a`
      <div class="reviews-row">
        <a
          class="reviews-link"
          href=${`${he(e.identifier)}#reviews`}
          target="_blank"
          rel="noopener"
          title=${o(`See reviews on archive.org`)}
          >${i} ${k(`chevron-right`,{size:14})}</a
        >
      </div>
    `}setlistTemplate(e){let t=e.setlist?{status:`loaded`,tracks:e.setlist}:this.setlistState;switch(t?.status){case`loading`:return this.setlistLoadingTemplate;case`failed`:return a`<p class="setlist-error">
          ${o(`Could not load the setlist.`)}
        </p>`;case`loaded`:return this.setlistRowsTemplate(t.tracks);default:return this.setlistRowsTemplate([])}}get setlistLoadingTemplate(){return a`
      <div class="setlist-loading" role="status" aria-live="polite">
        <span class="visually-hidden">${o(`Loading setlist`)}</span>
        <div class="placeholder-rows" aria-hidden="true">
          ${ee(m(Pt),()=>a`
              <div class="track">
                <lma-tombstone class="placeholder-number"></lma-tombstone>
                <lma-tombstone class="placeholder-title"></lma-tombstone>
              </div>
            `)}
        </div>
      </div>
    `}setlistRowsTemplate(e){return e.length===0?a`<p class="no-setlist">${o(`No setlist available`)}</p>`:a`
      <ol class="setlist" role="list">
        ${e.map((e,t)=>{let r=e.durationSecs>0?se(e.durationSecs):n;return a`
            <li class="track">
              <span class="track-number">${t+1}</span>
              <span class="track-title">${e.title}</span>
              <span class="track-duration">${r}</span>
              ${this.trackActionsTemplate(e,t)}
            </li>
          `})}
      </ol>
    `}handleQueueAllClick(){this.requestQueue(void 0)}handlePlayAllClick(){this.requestPlay(void 0)}requestQueue(e){let t=this.show;if(!t)return;let n=e===void 0?{show:t}:{show:t,trackIndex:e};this.dispatchEvent(N(A.showQueueRequested,n))}requestPlay(e){let t=this.show;if(!t)return;let n=e===void 0?{show:t}:{show:t,trackIndex:e};this.dispatchEvent(N(A.showPlayRequested,n))}handleCloseClick(){this.dispatchEvent(N(A.closeRequested,null))}handleDateClick(e){let t=this.show;!t||!X(e)||(e.preventDefault(),this.dispatchEvent(N(A.showSelected,{show:t})))}handleArtistClick(e,t){X(e)&&(e.preventDefault(),this.dispatchEvent(N(A.artistSelected,{artist:t})))}static get styles(){return e`
      :host {
        display: flex;
        flex-direction: column;
        position: relative;
        box-sizing: border-box;
        overflow: hidden;
        background: var(--ia-theme-secondary-background-color, #fff);
        color: var(--ia-theme-primary-text-color, #2c2c2c);
        font-family: var(--ia-theme-base-font-family, inherit);
        border-radius: 12px;
        box-shadow:
          0 25px 60px rgba(0, 0, 0, 0.18),
          0 0 0 1px rgba(0, 0, 0, 0.07);
      }

      .scroll {
        flex: 1 1 auto;
        min-height: 0;
        overflow-y: auto;
        padding-bottom: 12px;
      }

      .close {
        position: absolute;
        top: 12px;
        right: 12px;
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        border: 0;
        border-radius: 9999px;
        background: var(--ia-theme-primary-background-color, #fbfbfd);
        color: var(--ia-theme-secondary-text-color, #666);
        cursor: pointer;
        transition: background-color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
      }

      .close:hover {
        background: var(--ia-theme-primary-cta-border, #c5d1df);
      }

      .header {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 16px;
        padding: 24px 24px 16px;
      }

      .headlines {
        flex: 1 1 auto;
        min-width: 0;
      }

      .headline {
        display: block;
        font-size: 22px;
        font-weight: 700;
        letter-spacing: -0.5px;
        line-height: 1.2;
        color: var(--ia-theme-primary-text-color, #2c2c2c);
      }

      a.headline {
        text-decoration: none;
      }

      a.headline:hover {
        text-decoration: underline;
      }

      .by {
        margin-top: 8px;
        font-size: 14px;
        line-height: 20px;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .artist-link,
      .reviews-link {
        color: var(--ia-theme-link-color, #4b64ff);
        text-decoration: none;
      }

      .artist-link:hover,
      .reviews-link:hover {
        text-decoration: underline;
      }

      .rating {
        flex-shrink: 0;
        display: flex;
        flex-direction: column;
        align-items: flex-end;
        gap: 2px;
        padding-right: 32px;
        font-size: 12px;
        line-height: 16px;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .stars {
        display: flex;
        align-items: center;
        gap: 2px;
      }

      .star svg {
        display: block;
        color: var(--ia-theme-primary-cta-border, #c5d1df);
        fill: var(--ia-theme-primary-cta-border, #c5d1df);
      }

      /* Rating-star yellow: no theme token describes a rating accent. */
      .star.filled svg {
        color: #facc15;
        fill: #facc15;
      }

      .provenance {
        padding: 0 20px 8px;
      }

      .section-label {
        margin-bottom: 4px;
        font-size: 10px;
        line-height: 14px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.1em;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .provenance-text {
        display: -webkit-box;
        -webkit-box-orient: vertical;
        -webkit-line-clamp: 2;
        line-clamp: 2;
        overflow: hidden;
        font-size: 11px;
        line-height: 1.375;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .provenance-text + .section-label {
        margin-top: 6px;
      }

      .reviews-row {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        padding: 0 24px 12px;
        font-size: 14px;
        line-height: 20px;
      }

      .reviews-link {
        display: flex;
        align-items: center;
        gap: 4px;
      }

      .setlist-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        padding: 0 24px 8px;
      }

      .setlist-heading {
        margin: 0;
        font-size: 14px;
        line-height: 20px;
        font-weight: 700;
        color: var(--ia-theme-primary-text-color, #2c2c2c);
      }

      .setlist-actions {
        display: flex;
        align-items: center;
        gap: 8px;
      }

      .queue-all,
      .play-all {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 4px 12px;
        border-radius: 4px;
        font-family: inherit;
        font-size: 12px;
        line-height: 16px;
        cursor: pointer;
        transition: background-color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
      }

      .queue-all {
        border: 1px solid var(--ia-theme-primary-cta-border, #c5d1df);
        background: var(--ia-theme-secondary-background-color, #fff);
        color: var(--ia-theme-primary-text-color, #2c2c2c);
      }

      .queue-all:hover {
        background: var(--ia-theme-primary-background-color, #fbfbfd);
      }

      .play-all {
        border: 1px solid var(--ia-theme-primary-cta-fill, #194880);
        background: var(--ia-theme-primary-cta-fill, #194880);
        color: var(--ia-theme-primary-cta-text-color, #fff);
      }

      .play-all svg,
      .track-play svg {
        fill: currentColor;
      }

      .track-action {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        box-sizing: border-box;
        padding: 0;
        border-radius: 9999px;
        cursor: pointer;
        transition:
          border-color 0.15s cubic-bezier(0.4, 0, 0.2, 1),
          color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
      }

      .track-queue {
        width: 24px;
        height: 24px;
        border: 1px solid var(--ia-theme-primary-cta-border, #c5d1df);
        background: none;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .track-queue:hover {
        border-color: var(--ia-theme-link-color, #4b64ff);
        color: var(--ia-theme-link-color, #4b64ff);
      }

      .track-play {
        width: 28px;
        height: 28px;
        border: 0;
        background: var(--ia-theme-primary-cta-fill, #194880);
        color: var(--ia-theme-primary-cta-text-color, #fff);
      }

      .no-setlist,
      .setlist-error {
        margin: 0;
        padding: 12px 24px;
        font-size: 12px;
        line-height: 16px;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .placeholder-number {
        flex-shrink: 0;
        width: 20px;
        height: 12px;
      }

      .placeholder-title {
        flex: 1 1 auto;
        width: auto;
        height: 18px;
      }

      .setlist {
        max-height: 220px;
        overflow-y: auto;
        margin: 0;
        padding: 0;
        list-style: none;
      }

      .track {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 16px;
      }

      .track-number {
        flex-shrink: 0;
        width: 20px;
        text-align: right;
        font-size: 12px;
        line-height: 16px;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      .track-title {
        flex: 1 1 auto;
        min-width: 0;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        font-size: 13px;
        line-height: 18px;
        color: var(--ia-theme-primary-text-color, #2c2c2c);
      }

      .track-duration {
        flex-shrink: 0;
        font-size: 12px;
        line-height: 16px;
        font-variant-numeric: tabular-nums;
        color: var(--ia-theme-secondary-text-color, #666);
      }

      /* http://webaim.org/techniques/css/invisiblecontent/ */
      .visually-hidden {
        position: absolute;
        clip: rect(0, 0, 0, 0);
        width: 1px;
        height: 1px;
        overflow: hidden;
      }
    `}};y([t({type:Object})],Z.prototype,`show`,void 0),y([t({attribute:!1})],Z.prototype,`setlistState`,void 0),y([t({type:Object})],Z.prototype,`capabilities`,void 0),Z=y([h(`lma-show-hover-card`),i()],Z);function Ft(e,t){return e?.top===t.top&&e.left===t.left&&e.right===t.right}var Q=class extends c{constructor(...e){super(...e),this.capabilities=j,this.cardHeight=0,this.via=`pointer`,this.focusCardOnRender=!1,this.cardResizeObserver=new ResizeObserver(e=>{let t=e.find(({target:e})=>e===this.observedCard)?.borderBoxSize[0]?.blockSize??0;t>0&&t!==this.cardHeight&&(this.cardHeight=t)}),this.scheduleClose=()=>{this.cancelClose(),this.closeTimer=setTimeout(()=>{this.closeTimer=void 0,this.close()},150)},this.cancelClose=()=>{this.closeTimer!==void 0&&(clearTimeout(this.closeTimer),this.closeTimer=void 0)},this.handleKeydown=e=>{let{key:t}=e;if(t!==`ArrowDown`&&t!==`Escape`||e.altKey||e.ctrlKey||e.metaKey||e.shiftKey)return;let n=this.card,r=this.opener;if(!(!this.cardOpen||!n||!r)&&e.composedPath().includes(r)){if(e.preventDefault(),t===`Escape`){this.close();return}n.focusFirstControl()}},this.handlePointerDown=e=>{let t=this.card;if(!this.cardOpen||!t)return;let n=e.composedPath();n.includes(t)||this.opener&&n.includes(this.opener)||this.close()},this.handleScroll=()=>{this.cardOpen&&this.close()},this.handleWheel=e=>{let t=this.card;!this.cardOpen||!t||e.composedPath().includes(t)||this.close()},this.handleResize=()=>{this.cardOpen&&this.close()}}open(e,t,n={}){let{via:r=`pointer`,opener:i}=n,a=this.show?.identifier===e.identifier;return a&&r===`button`&&i&&i===this.opener?(this.focusInCard()?this.closeAndRestoreFocus():this.close(),!1):(this.cancelClose(),this.via=r,this.opener=i,this.focusCardOnRender=r===`button`,a?(Ft(this.anchorRect,t)?this.focusCardOnRender&&this.requestUpdate():this.anchorRect=t,!1):(this.artist=void 0,this.aboutState=void 0,this.show=e,this.anchorRect=t,this.setlistState=void 0,this.cardHeight=0,!0))}openArtist(e,t,n={}){let{via:r=`pointer`,opener:i}=n,a=this.artist?.identifier===e.identifier;return this.cancelClose(),this.via=r,this.opener=i,this.focusCardOnRender=!1,a?(this.artist=e,Ft(this.anchorRect,t)||(this.anchorRect=t),!1):(this.show=void 0,this.setlistState=void 0,this.artist=e,this.anchorRect=t,this.aboutState=void 0,this.cardHeight=0,!0)}setArtistAboutState(e,t){this.artist?.identifier===e&&(this.aboutState=t)}endArtistPreview(e){this.artist?.identifier===e.identifier&&this.scheduleClose()}resumeArtistPreview(e){this.artist?.identifier===e.identifier&&this.cancelClose()}setSetlistState(e,t){this.show?.identifier===e&&(this.setlistState=t)}endPreview(e){this.show?.identifier===e.identifier&&this.scheduleClose()}resumePreview(e){this.show?.identifier===e.identifier&&this.cancelClose()}close(){this.cancelClose(),this.show=void 0,this.artist=void 0,this.anchorRect=void 0,this.setlistState=void 0,this.aboutState=void 0,this.cardHeight=0,this.via=`pointer`,this.opener=void 0,this.focusCardOnRender=!1}connectedCallback(){super.connectedCallback(),window.addEventListener(`scroll`,this.handleScroll,{capture:!0,passive:!0}),window.addEventListener(`wheel`,this.handleWheel,{capture:!0,passive:!0}),window.addEventListener(`pointerdown`,this.handlePointerDown,{capture:!0,passive:!0}),window.addEventListener(`keydown`,this.handleKeydown,{capture:!0}),window.addEventListener(`resize`,this.handleResize)}disconnectedCallback(){super.disconnectedCallback(),window.removeEventListener(`scroll`,this.handleScroll,{capture:!0}),window.removeEventListener(`wheel`,this.handleWheel,{capture:!0}),window.removeEventListener(`pointerdown`,this.handlePointerDown,{capture:!0}),window.removeEventListener(`keydown`,this.handleKeydown,{capture:!0}),window.removeEventListener(`resize`,this.handleResize),this.close(),this.observeCard(void 0)}render(){let e=this.anchorRect;if(!this.cardOpen||!e)return n;let t=document.documentElement,r=this.show,i=Mt(e,{width:t.clientWidth,height:t.clientHeight},this.cardHeight,r?500:420),o=u({top:`${i.top}px`,left:`${i.left}px`,width:`${i.width}px`,maxHeight:`${i.maxHeight}px`});return r?a`${l(r.identifier,a`
        <lma-show-hover-card
          class="card ${i.side}"
          style=${o}
          .show=${r}
          .setlistState=${this.setlistState}
          .capabilities=${this.capabilities}
          @pointerenter=${this.cancelClose}
          @pointerleave=${this.handleCardPointerLeave}
          @focusin=${this.cancelClose}
          @keydown=${this.handleCardKeydown}
          @closeRequested=${this.handleCloseRequested}
          @showSelected=${this.handleShowSelected}
          @artistSelected=${this.handleArtistSelected}
          @showPlayRequested=${this.handleShowPlayRequested}
          @showQueueRequested=${this.handleShowQueueRequested}
        ></lma-show-hover-card>
      `)}`:this.artistCardTemplate(i.side,o)}artistCardTemplate(e,t){let r=this.artist;return r?a`${l(r.identifier,a`
        <lma-artist-info-card
          class="card ${e}"
          style=${t}
          .artist=${r}
          .aboutState=${this.aboutState}
          @pointerenter=${this.cancelClose}
          @pointerleave=${this.handleCardPointerLeave}
          @focusin=${this.cancelClose}
          @keydown=${this.handleCardKeydown}
          @closeRequested=${this.handleCloseRequested}
        ></lma-artist-info-card>
      `)}`:n}updated(){let e=this.card;this.observeCard(e),this.focusCardOnRender&&e&&(this.focusCardOnRender=!1,this.focusCardWhenRendered(e))}async focusCardWhenRendered(e){await e.updateComplete,this.card===e&&e.focusFirstControl()}observeCard(e){e!==this.observedCard&&(this.cardResizeObserver.disconnect(),this.observedCard=e,e&&this.cardResizeObserver.observe(e))}get cardOpen(){return!!this.show||!!this.artist}focusInCard(){return!!this.card?.matches(`:focus-within`)}closeAndRestoreFocus(){let e=this.opener;e?.isConnected&&e.restorePreviewFocus(),this.close()}handleCardPointerLeave(e){this.via!==`pointer`||e.pointerType!==`mouse`||this.scheduleClose()}handleCardKeydown(e){if(e.altKey||e.ctrlKey||e.metaKey)return;if(e.key===`Escape`){e.preventDefault(),this.closeAndRestoreFocus();return}if(e.key!==`Tab`)return;let t=this.card?.focusableControls()??[],n=e.shiftKey?t[0]:t[t.length-1];!n||e.composedPath()[0]!==n||(this.closeAndRestoreFocus(),e.shiftKey&&e.preventDefault())}handleCloseRequested(){if(this.focusInCard()){this.closeAndRestoreFocus();return}this.close()}handleShowSelected(e){this.close(),F(this,A.showSelected,e)}handleArtistSelected(e){this.close(),F(this,A.artistSelected,e)}handleShowPlayRequested(e){F(this,A.showPlayRequested,e)}handleShowQueueRequested(e){F(this,A.showQueueRequested,e)}static get styles(){return[R,e`
        :host {
          position: fixed;
          inset: 0;
          z-index: var(--lma-z-floater);
          pointer-events: none;
        }

        .card {
          position: absolute;
          pointer-events: auto;
        }
      `]}};y([t({type:Object})],Q.prototype,`capabilities`,void 0),y([s()],Q.prototype,`show`,void 0),y([s()],Q.prototype,`anchorRect`,void 0),y([s()],Q.prototype,`setlistState`,void 0),y([s()],Q.prototype,`artist`,void 0),y([s()],Q.prototype,`aboutState`,void 0),y([s()],Q.prototype,`cardHeight`,void 0),y([g(`.card`)],Q.prototype,`card`,void 0),Q=y([h(`lma-floater-host`)],Q);var It=250,Lt=250,Rt=`https://archive.org/details/etree`,zt=`https://archive.org/create.php?lma=1`,$=class extends c{constructor(...e){super(...e),this.capabilities={...j,playbackAvailable:!0,showMoreSections:[`artists`,`shows`]},this.surfaceStack=new be(this),this.mode=`desktop`,this.modeReady=!1,this.servicesReady=!1,this.heroSearchQuery=``,this.railSearchQuery=``,this.dockRequestGeneration=0,this.queueTouched=!1,this.dockPlaying=!1,this.dockBuffering=!1,this.dockCurrentTimeSecs=0,this.dockDurationSecs=0,this.dockDurationFixed=!1,this.playRequestPending=!1,this.playQueue=new it(e=>this.handleQueueTransition(e)),this.recentlyPlayed=new at(tt()),this.dockRecentlyPlayed=[],this.recentlyPlayedDirty=!1,this.recentlyPlayedRestoreSettled=!1,this.dockTrackCount=0,this.dockShowCount=0,this.dockCanPrevious=!1,this.dockCanNext=!1,this.dockPlayOrder=ue,this.sessionFavoritedShows=[],this.sessionUnfavoritedShowIds=new Set,this.queueFavoriteShowIds=new Set,this.favoriteCheckedShowIds=new Set,this.favoriteToggledShowIds=new Set,this.favoriteSavingShowIds=new Set,this.volumeLevel=me,this.volumeChosen=!1,this.volumeReadOnly=!this.detectVolumeSettable(),this.sectionOrder=O,this.activeEdgeScrollDirection=0,this.handleDocumentDragOver=e=>{if(this.updateEdgeScroll(e.clientY),this.dropTarget===void 0)return;e.preventDefault();let t=e.dataTransfer;t&&(t.dropEffect=`move`)},this.handleDocumentDrop=e=>{let t=this.dragSectionId,n=this.dropTarget;t===void 0||n===void 0||(e.preventDefault(),this.landSectionDrop(t,n.section,n.before))},this.handleWindowMouseUp=()=>{this.disarmSection()},this.setupStarted=!1,this.isQueued=e=>this.playQueue.hasTrackFile(e.show.identifier,e.track.fileName),this.handlePageHide=()=>{this.flushVolumePersist(),this.flushQueuePersist()}}render(){let e=this.modeReady?a`${this.heroTemplate} ${this.sectionFramesTemplate}`:n,t=this.modeReady?this.playerDockTemplate:n,r=this.surfaceStack.hasOpenSurface;return a`
      <div id="scene-root" class=${this.mode}>
        <div id="page-container" ?inert=${r}>${e}</div>
        ${this.showMoreSurfaceTemplate} ${this.infoSurfaceTemplate}
        ${t}
        <lma-floater-host
          .capabilities=${this.capabilities}
          @showSelected=${this.handleShowSelected}
          @artistSelected=${this.handleArtistSelected}
          @showPlayRequested=${this.handleShowPlayRequested}
          @showQueueRequested=${this.handleShowQueueRequested}
        ></lma-floater-host>
      </div>
    `}get showMoreSurfaceTemplate(){let e=this.showMoreSection;if(!e)return n;let t=D()[e];return a`
      <lma-overlay
        tier="section"
        label=${t}
        disableBackdropClose
        @closeRequested=${this.handleShowMoreCloseRequested}
      >
        <lma-show-more-surface
          section=${e}
          heading=${t}
          ?servicesLoading=${!this.servicesReady}
          mode=${this.mode}
          query=${this.heroSearchQuery}
          .capabilities=${this.capabilities}
          .searchAdapter=${this.searchAdapter}
          .artistCounts=${this.artistCounts}
          .resizeObserver=${this.resizeObserver}
          @closeRequested=${this.handleShowMoreCloseRequested}
          @contentScrolled=${this.handleShowMoreContentScrolled}
          @searchQueryChanged=${this.handleShowMoreSearchQueryChanged}
          @artistSelected=${this.handleArtistSelected}
          @artistHovered=${this.handleArtistHovered}
          @artistHoverEnded=${this.handleArtistHoverEnded}
          @showSelected=${this.handleShowSelected}
          @showPlayRequested=${this.handleShowPlayRequested}
          @showQueueRequested=${this.handleShowQueueRequested}
          @showPreviewRequested=${this.handleShowPreviewRequested}
          @showPreviewResumed=${this.handleShowPreviewResumed}
          @showPreviewEnded=${this.handleShowPreviewEnded}
        ></lma-show-more-surface>
      </lma-overlay>
    `}get infoSurfaceTemplate(){let e=this.infoSurface;if(!e)return n;let t=e===`history`,r=o(t?`History of the LMA`:`The Taper Section`),i=t?a`<lma-history-surface
          mode=${this.mode}
          @closeRequested=${this.handleInfoSurfaceCloseRequested}
          @taperSectionRequested=${this.handleTaperSectionRequested}
        ></lma-history-surface>`:a`<lma-taper-section-surface
          mode=${this.mode}
          .tapingStats=${this.tapingStats??B}
          @closeRequested=${this.handleInfoSurfaceCloseRequested}
          @lmaHistoryRequested=${this.handleLMAHistoryRequested}
        ></lma-taper-section-surface>`;return a`
      <lma-overlay
        class="info-overlay"
        tier="section"
        label=${r}
        disableBackdropClose
        @closeRequested=${this.handleInfoSurfaceCloseRequested}
      >
        ${i}
      </lma-overlay>
    `}get playerDockTemplate(){return ne(async()=>{await _(()=>v(()=>import(`./lma-player-dock-71YGSZvs.js`),__vite__mapDeps([0,1,2,3,4,5,6,7,8,9,10,11,12,13])))},()=>a`
          <lma-player-dock
            mode=${this.mode}
            .show=${this.dockShow}
            .track=${this.dockTrack}
            ?playing=${this.dockPlaying}
            ?buffering=${this.dockBuffering||this.playRequestPending}
            .error=${this.dockError}
            .queueCount=${this.dockTrackCount}
            .showCount=${this.dockShowCount}
            .currentTimeSecs=${this.dockCurrentTimeSecs}
            .durationSecs=${this.dockDurationSecs}
            ?canPrevious=${this.dockCanPrevious&&!this.playRequestPending}
            ?canNext=${this.dockCanNext&&!this.playRequestPending}
            .playOrder=${this.dockPlayOrder}
            .recentlyPlayed=${this.dockRecentlyPlayed}
            .volumeLevel=${this.volumeLevel}
            ?volumeReadOnly=${this.volumeReadOnly}
            .favoriteShowIds=${this.queueFavoriteShowIds}
            @playPauseToggled=${this.handleDockPlayPauseToggled}
            @previousTrackRequested=${this.handleDockPreviousRequested}
            @nextTrackRequested=${this.handleDockNextRequested}
            @trackJumpRequested=${this.handleDockTrackJumpRequested}
            @showPlayRequested=${this.handleShowPlayRequested}
            @seekRequested=${this.handleDockSeekRequested}
            @volumeRequested=${this.handleDockVolumeRequested}
            @queueEntryMoveRequested=${this.handleQueueEntryMoveRequested}
            @queueEntryRemoveRequested=${this.handleQueueEntryRemoveRequested}
            @queueShowRemoveRequested=${this.handleQueueShowRemoveRequested}
            @showFavoriteToggleRequested=${this.handleShowFavoriteToggleRequested}
          ></lma-player-dock>
        `)}get heroTemplate(){return a`
      ${this.heroLinksTemplate}
      <header class="hero">
        <div class="hero-main">
          <div class="hero-title-row">
            <img class="hero-logo" src=${oe} alt="" />
            <h1 class="hero-heading">
              ${o(a`<span class="initial">L</span>ive
                  <span class="initial">M</span>usic
                  <span class="initial">A</span>rchive`)}
            </h1>
          </div>
          <p class="hero-tagline">
            ${o(`Search the Live Music Archive for recordings from live shows since 1964.`)}
          </p>
          <ia-clearable-text-input
            class="hero-search"
            .value=${this.heroSearchQuery}
            placeholder=${o(`Enter an Artist, Date, Venue, Taper, Recording Medium....`)}
            screenReaderLabel=${o(`Search the Live Music Archive`)}
            clearButtonScreenReaderLabel=${o(`Clear search query`)}
            @input=${this.handleSearchInput}
            @keydown=${this.handleSearchKeydown}
          ></ia-clearable-text-input>
        </div>
        ${this.concertBrowserEntryTemplate}
      </header>
    `}get heroLinksTemplate(){let e=this.mode===`mobile`?o(`Upload`):o(`Upload Recording`);return a`
      <div class="hero-links">
        <div class="hero-link-group">
          <button
            class="hero-link history-link"
            title=${o(`History of the LMA — half a century of tapes, networks, and the people who kept both`)}
            @click=${this.handleHistoryLinkClick}
          >
            ${o(`LMA History`)}
          </button>
          <button
            class="hero-link tapers-link"
            title=${o(`About Tapers — the tapes, the gear, and the people who kept the music`)}
            @click=${this.handleTapersLinkClick}
          >
            ${o(`About Tapers`)}
          </button>
        </div>
        <div class="hero-link-group actions">
          <a
            class="classic-mode-link"
            href=${Rt}
            target="_blank"
            rel="noopener"
            title=${o(`Classic mode — the original Live Music Archive on archive.org`)}
          >
            ${o(`Classic Mode`)}
          </a>
          <a
            class="upload-link"
            href=${zt}
            target="_blank"
            rel="noopener"
            title=${o(`Upload Recording`)}
          >
            ${k(`upload`,{size:16})}
            <span>${e}</span>
          </a>
        </div>
      </div>
    `}get concertBrowserEntryTemplate(){let e=this.capabilities.concertBrowserAvailable;return this.mode===`desktop`?a`
      <div class="concert-browser-entry">
        ${e?a`<span class="concert-browser-title"
          >${o(`LMA Concert Browser`)}</span
        >`:n}
        <img
          class="concert-browser-image"
          src=${xe}
          alt=""
        />
      </div>
    `:e?a`
        <div class="concert-browser-entry compact">
          <button class="concert-browser-open">
            ${o(`Open LMA Concert Browser`)}
          </button>
        </div>
      `:n}get sectionFramesTemplate(){return a`
      <div id="sections">
        ${p(this.sectionOrder.filter(e=>this.sectionShown(e)),e=>e,e=>this.sectionSlotTemplate(e))}
      </div>
    `}sectionShown(e){return e===`favorites`?this.favoritesCollection!=null:!0}sectionSlotTemplate(e){let t=this.dropTarget,r=t!==void 0&&t.section===e,i=r&&t.before,o=r&&!t.before;return a`
      <div
        class=${f({"section-slot":!0,"drag-source":this.dragSectionId===e})}
        draggable=${this.armedSectionId===e?`true`:`false`}
        @dragstart=${t=>this.handleSectionDragStart(t,e)}
        @dragend=${this.handleSectionDragEnd}
        @dragover=${t=>this.handleSectionDragOver(t,e)}
        @drop=${t=>this.handleSectionDrop(t,e)}
      >
        ${i?this.dropLineTemplate(!0):n}
        ${this.sectionTemplate(e,D()[e])}
        ${o?this.dropLineTemplate(!1):n}
      </div>
    `}dropLineTemplate(e){return a`
      <div class="drop-line ${e?`before`:`after`}">
        <div class="drop-line-dot"></div>
      </div>
    `}sectionTemplate(e,t){return e===`artists`?this.artistsSectionTemplate(t):this.showRailTemplate(e,t)}artistsSectionTemplate(e){return a`
      <lma-artists-section
        heading=${e}
        ?servicesLoading=${!this.servicesReady}
        mode=${this.mode}
        query=${this.railSearchQuery}
        .capabilities=${this.capabilities}
        .searchAdapter=${this.searchAdapter}
        .artistCounts=${this.artistCounts}
        @artistSelected=${this.handleArtistSelected}
        @artistHovered=${this.handleArtistHovered}
        @artistHoverEnded=${this.handleArtistHoverEnded}
        @reorderGripPressed=${this.handleReorderGripPressed}
        @reorderGripReleased=${this.handleReorderGripReleased}
        @showMoreRequested=${this.handleShowMoreRequested}
      ></lma-artists-section>
    `}showRailTemplate(e,t){return a`
      <lma-show-rail-section
        sectionId=${e}
        heading=${t}
        ?servicesLoading=${!this.servicesReady}
        mode=${this.mode}
        query=${this.railSearchQuery}
        .capabilities=${this.capabilities}
        .searchAdapter=${this.searchAdapter}
        .userState=${this.userState}
        .favoritesCollection=${this.favoritesCollection}
        .favoritedShows=${this.sessionFavoritedShows}
        .unfavoritedShowIds=${this.sessionUnfavoritedShowIds}
        @showSelected=${this.handleShowSelected}
        @showPlayRequested=${this.handleShowPlayRequested}
        @showQueueRequested=${this.handleShowQueueRequested}
        @showPreviewRequested=${this.handleShowPreviewRequested}
        @showPreviewResumed=${this.handleShowPreviewResumed}
        @showPreviewEnded=${this.handleShowPreviewEnded}
        @reorderGripPressed=${this.handleReorderGripPressed}
        @reorderGripReleased=${this.handleReorderGripReleased}
        @showMoreRequested=${this.handleShowMoreRequested}
      ></lma-show-rail-section>
    `}updated(e){e.has(`appServices`)&&this.setupAppServicesDependents(),this.registerPlayerDockExemption(),this.registerFloaterHostExemption(),e.has(`showMoreSection`)&&this.floaterHost?.close()}registerFloaterHostExemption(){let e=this.showMoreSection?this.floaterHost:void 0;if(e===this.exemptFloaterHost)return;let t=this.exemptFloaterHost;t&&this.surfaceStack.unregisterExemptRoot(t),e&&this.surfaceStack.registerExemptRoot(e),this.exemptFloaterHost=e}registerPlayerDockExemption(){let e=this.playerDock??void 0;if(e===this.exemptPlayerDock)return;let t=this.exemptPlayerDock;t&&this.surfaceStack.unregisterExemptRoot(t),e&&this.surfaceStack.registerExemptRoot(e),this.exemptPlayerDock=e}async handleShowMoreRequested(e){let t=e.detail.section;if(!(t!==`artists`&&t!==`shows`)&&this.capabilities.showMoreSections.includes(t)){try{await _(()=>v(()=>import(`./lma-show-more-surface-RIe1yqij.js`),__vite__mapDeps([14,1,2,3,4,5,6,7,15,16,10,11,12,13,17,18]))),await _(()=>v(()=>import(`./lma-overlay-Rrt46Gde.js`),__vite__mapDeps([19,1,2,3,4,5,6,7,20,13])))}catch{return}this.showMoreSection=t}}handleShowMoreCloseRequested(){this.showMoreSurface?.applyPendingSearch(),this.showMoreSection=void 0}handleShowMoreContentScrolled(){this.floaterHost?.close()}handleShowMoreSearchQueryChanged(e){this.applySearch(e.detail.query)}handleHistoryLinkClick(e){this.focusClickedControl(e),this.openInfoSurface(`history`)}handleTapersLinkClick(e){this.focusClickedControl(e),this.openInfoSurface(`taperSection`)}focusClickedControl(e){let t=e.currentTarget;t instanceof HTMLElement&&t.focus()}async openInfoSurface(e){try{e===`history`?await _(()=>v(()=>import(`./lma-history-surface-u0r9Ya6t.js`),__vite__mapDeps([21,1,2,3,4,5,6,7,8,11,22,13]))):await _(()=>v(()=>import(`./lma-taper-section-surface-C5XeIC57.js`),__vite__mapDeps([23,1,2,3,4,5,6,7,24,11,10,22,13]))),await _(()=>v(()=>import(`./lma-overlay-Rrt46Gde.js`),__vite__mapDeps([19,1,2,3,4,5,6,7,20,13])))}catch{return}this.infoSurface=e}handleTaperSectionRequested(){this.swapInfoSurface(`taperSection`)}handleLMAHistoryRequested(){this.swapInfoSurface(`history`)}async swapInfoSurface(e){await this.openInfoSurface(e),this.infoSurface===e&&(await this.updateComplete,this.infoOverlay?.focusSurface())}handleInfoSurfaceCloseRequested(){this.infoSurface=void 0}willUpdate(e){super.willUpdate(e),this.showMoreSection||(this.railSearchQuery=this.heroSearchQuery),(e.has(`infoSurface`)||e.has(`searchAdapter`)||e.has(`servicesReady`))&&this.infoSurface===`taperSection`&&this.loadTapingStats()}loadTapingStats(){let e=this.tapingStats?.status;if(e===`loaded`||e===`loading`)return;let t=this.searchAdapter;if(!t){this.servicesReady&&(this.tapingStats={status:`failed`});return}this.tapingStats=B,ve(t).then(e=>{this.tapingStats={status:`loaded`,stats:e}},()=>{this.tapingStats={status:`failed`}})}connectedCallback(){super.connectedCallback(),this.observeSceneRoot(),window.addEventListener(`pagehide`,this.handlePageHide)}disconnectedCallback(){super.disconnectedCallback(),window.removeEventListener(`pagehide`,this.handlePageHide),this.disconnectResizeObserver(),this.clearSearchDebounce(),this.endSectionDrag(),this.flushVolumePersist(),this.exemptPlayerDock=void 0,this.exemptFloaterHost=void 0,this.flushQueuePersist(),this.playbackEngine?.dispose(),this.playbackEngine=void 0,this.loadedEngineTrack=void 0,this.dockPlaying=!1,this.dockBuffering=!1,this.playRequestPending=!1,this.dockError=void 0,this.dockCurrentTimeSecs=0}handleReorderGripPressed(e){this.armedSectionId=e.detail.section,window.addEventListener(`mouseup`,this.handleWindowMouseUp)}handleReorderGripReleased(){this.disarmSection()}disarmSection(){window.removeEventListener(`mouseup`,this.handleWindowMouseUp),this.armedSectionId=void 0}handleSectionDragStart(e,t){if(this.armedSectionId!==t)return;window.removeEventListener(`mouseup`,this.handleWindowMouseUp),this.dragSectionId=t;let n=e.dataTransfer;n&&(n.effectAllowed=`move`,n.setData(`text/plain`,t)),document.addEventListener(`dragover`,this.handleDocumentDragOver),document.addEventListener(`drop`,this.handleDocumentDrop)}handleSectionDragOver(e,t){let n=this.dragSectionId;if(n===void 0)return;e.preventDefault();let r=e.dataTransfer;if(r&&(r.dropEffect=`move`),n===t){this.dropTarget=void 0;return}let i=e.currentTarget;if(!(i instanceof HTMLElement))return;let a=i.getBoundingClientRect(),o=vt(e.clientY,a.top,a.height),s=this.dropTarget;s!==void 0&&s.section===t&&s.before===o||(this.dropTarget={section:t,before:o})}handleSectionDrop(e,t){let n=this.dragSectionId;if(n===void 0)return;e.preventDefault();let r=this.dropTarget,i=r!==void 0&&r.section===t?r.before:!0;this.landSectionDrop(n,t,i)}landSectionDrop(e,t,n){if(e!==t){let r=yt(this.sectionOrder,e,t,n);this.sameShownOrder(r)||(this.sectionOrder=r,this.persistSectionOrder())}this.endSectionDrag()}sameShownOrder(e){let t=e=>e.filter(e=>this.sectionShown(e)),n=t(this.sectionOrder),r=t(e);return n.length===r.length&&n.every((e,t)=>e===r[t])}persistSectionOrder(){this.userState?.saveSectionOrder([...this.sectionOrder])}handleSectionDragEnd(){this.endSectionDrag()}endSectionDrag(){document.removeEventListener(`dragover`,this.handleDocumentDragOver),document.removeEventListener(`drop`,this.handleDocumentDrop),this.stopEdgeScroll(),this.disarmSection(),this.dragSectionId=void 0,this.dropTarget=void 0}updateEdgeScroll(e){let t=_t(e,window.innerHeight);if(t===0){this.stopEdgeScroll();return}if(this.activeEdgeScrollDirection=t,this.edgeScrollFrame!==void 0)return;let n=()=>{if(this.activeEdgeScrollDirection===0){this.edgeScrollFrame=void 0;return}window.scrollBy(0,this.activeEdgeScrollDirection*22),this.edgeScrollFrame=window.requestAnimationFrame(n)};this.edgeScrollFrame=window.requestAnimationFrame(n)}stopEdgeScroll(){this.activeEdgeScrollDirection=0;let e=this.edgeScrollFrame;e!==void 0&&(window.cancelAnimationFrame(e),this.edgeScrollFrame=void 0)}handleSearchInput(e){let t=e.currentTarget;t instanceof b&&this.scheduleSearch(t.value)}handleSearchKeydown(e){if(e.isComposing||e.key!==`Enter`)return;let t=e.currentTarget;t instanceof b&&this.applySearch(t.value)}scheduleSearch(e){this.clearSearchDebounce(),this.searchDebounceTimer=window.setTimeout(()=>{this.applySearch(e)},350)}applySearch(e){this.clearSearchDebounce(),this.heroSearchQuery=e}clearSearchDebounce(){this.searchDebounceTimer!==void 0&&(window.clearTimeout(this.searchDebounceTimer),this.searchDebounceTimer=void 0)}handleShowPreviewRequested(e){let t=this.floaterHost;if(!t)return;let{show:n,anchorRect:r,via:i,opener:a}=e.detail;t.open(n,r,{via:i,opener:a})&&!n.setlist&&this.loadPreviewSetlist(t,n)}async loadPreviewSetlist(e,t){let n=this.showDetails;if(!n)return;let{identifier:r}=t;e.setSetlistState(r,de);try{let t=await n.getShowDetails(r);e.setSetlistState(r,{status:`loaded`,tracks:t.setlist,source:t.source,lineage:t.lineage})}catch(t){ae(t,{extra:{identifier:r,method:`loadPreviewSetlist`}}),e.setSetlistState(r,pe)}}handleShowPreviewResumed(e){this.floaterHost?.resumePreview(e.detail.show)}handleShowPreviewEnded(e){this.floaterHost?.endPreview(e.detail.show)}handleArtistHovered(e){let t=this.floaterHost;if(!t)return;let{artist:n,anchorRect:r,via:i,opener:a,resumed:o}=e.detail;if(o){t.resumeArtistPreview(n);return}t.openArtist(n,r,{via:i,opener:a})&&this.loadArtistAbout(t,n)}async loadArtistAbout(e,t){let n=this.artistAbout;if(!n)return;let{identifier:r}=t;e.setArtistAboutState(r,fe);let i=await n.about(t.name);e.setArtistAboutState(r,i)}handleArtistHoverEnded(e){this.floaterHost?.endArtistPreview(e.detail.artist)}handleShowSelected(e){this.navigateToUrl(I(e.detail.show.identifier))}ensurePlaybackEngine(){if(this.playbackEngine)return this.playbackEngine;let e=this.createPlaybackEngine();return e.on(`playingChanged`,e=>{this.dockPlaying=e,e&&(this.dockError=void 0)}),e.on(`bufferingChanged`,e=>{this.dockBuffering=e}),e.on(`playbackStarted`,e=>{this.markStartedTrackPlayed(e)}),e.on(`timeChanged`,e=>{this.dockCurrentTimeSecs=e}),e.on(`durationChanged`,e=>{let t=Number.isFinite(e)&&e>0;this.dockDurationFixed||!t||(this.dockDurationSecs=e,this.dockDurationFixed=!0)}),e.on(`errorChanged`,e=>{if(!e){this.dockError?.kind===`track`&&(this.dockError=void 0);return}let t=this.loadedEngineTrack?.title;t&&(this.dockError={kind:`track`,name:t})}),e.on(`trackEnded`,e=>{this.advancePastEndedTrack(e)}),e.volume=this.volumeLevel,this.playbackEngine=e,e}detectVolumeSettable(){return gt()}markStartedTrackPlayed(e){let t=this.loadedEngineTrack?.url,n=this.playQueue.current?.track.url;e!==t||e!==n||this.playQueue.markCurrentPlayed()&&this.commitQueueChange()}advancePastEndedTrack(e){let t=this.loadedEngineTrack;if(!t||t.url!==e)return;let n=this.playQueue.advancePast(e,{emptyAtEnd:!0});if(n.outcome===`emptied`){this.stopForEmptiedQueue();return}this.advanceToQueuedTrack(n,!1)}handleDockNextRequested(){if(this.playRequestPending)return;let e=this.playQueue.current;if(!e)return;let t=this.ensurePlaybackEngine(),n=t.isPlaying||t.isBuffering;this.advanceToQueuedTrack(this.playQueue.advancePast(e.track.url),n)}handleDockPreviousRequested(){if(this.playRequestPending)return;let e=this.playQueue.stepBack();if(e.outcome!==`retreated`){this.pullBackFromHistory();return}let t=this.ensurePlaybackEngine(),n=t.isPlaying||t.isBuffering;this.loadQueuedTrack(e.entry,n)}pullBackFromHistory(){let e=this.recentlyPlayed.takeNewestPullable(this.isQueued);if(!e)return;this.recentlyPlayedDirty=!0;let t=!this.playQueue.current,n=this.playQueue.pullBack(e);if(!n)return;let r=this.ensurePlaybackEngine(),i=t||r.isPlaying||r.isBuffering;this.loadQueuedTrack(n,i)}handleQueueTransition(e){e.kind!==`trackLeft`||!e.entry.played||(this.recentlyPlayed.record(e.entry),this.recentlyPlayedDirty=!0)}handleDockTrackJumpRequested(e){if(this.playRequestPending)return;let t=this.playQueue.jumpTo(e.detail.position);t&&this.loadQueuedTrack(t,!0)}handleQueueEntryMoveRequested(e){if(this.playRequestPending)return;let{from:t,to:n}=e.detail;this.playQueue.moveEntry(t,n)&&this.commitQueueChange()}handleQueueEntryRemoveRequested(e){this.playRequestPending||this.applyQueueRemoval(this.playQueue.removeEntry(e.detail.position))}handleQueueShowRemoveRequested(e){this.playRequestPending||this.applyQueueRemoval(this.playQueue.removeRun(e.detail.position))}applyQueueRemoval(e){e.outcome===`kept`?this.commitQueueChange():e.outcome===`replaced`?this.showReplacementEntry(e.entry):e.outcome===`emptied`&&this.stopForEmptiedQueue()}showReplacementEntry(e){let t=this.playbackEngine;if(t){let n=t.isPlaying||t.isBuffering;this.loadQueuedTrack(e,n);return}this.displayTrackInDock(e.show,e.track),this.dockError=void 0,this.commitQueueChange()}stopForEmptiedQueue(){this.playbackEngine?.dispose(),this.playbackEngine=void 0,this.loadedEngineTrack=void 0,this.dockShow=void 0,this.dockTrack=void 0,this.dockPlaying=!1,this.dockBuffering=!1,this.dockError=void 0,this.dockCurrentTimeSecs=0,this.dockDurationSecs=0,this.dockDurationFixed=!1,this.commitQueueChange()}handleShowFavoriteToggleRequested(e){let t=e.detail.show,n=!this.queueFavoriteShowIds.has(t.identifier);if(n&&this.favoritesCollection===null){this.favoriteAfterSignIn(t);return}this.saveShowFavorite(t,n)}favoriteAfterSignIn(e){this.openLoginModal(async()=>{await this.handleSignedIn(),await this.saveShowFavorite(e,!0)})}async saveShowFavorite(e,t){let n=this.userState;if(!n)return;let r=e.identifier;if(this.favoriteSavingShowIds.has(r))return;this.favoriteCheckedShowIds.add(r),this.favoriteToggledShowIds.add(r),this.favoriteSavingShowIds.add(r);let i=this.sessionFavoritedShows.some(e=>e.identifier===r),a=this.sessionUnfavoritedShowIds.has(r);this.setQueueFavorite(r,t),this.setRailFavoriteState(e,t,!t);try{(t?await n.addFavorite(r,{title:e.title}):await n.removeFavorite(r)).success||(this.setQueueFavorite(r,!t),this.setRailFavoriteState(e,i,a))}finally{this.favoriteSavingShowIds.delete(r)}}setRailFavoriteState(e,t,n){let r=e.identifier,i=this.sessionFavoritedShows.filter(e=>e.identifier!==r);this.sessionFavoritedShows=t?[e,...i]:i;let a=new Set(this.sessionUnfavoritedShowIds);n?a.add(r):a.delete(r),this.sessionUnfavoritedShowIds=a}syncQueueFavorites(){let e=this.userState;if(!e)return;let t=new Set;for(let e of this.dockPlayOrder.entries){let n=e.show.identifier;this.favoriteCheckedShowIds.has(n)||(this.favoriteCheckedShowIds.add(n),t.add(n))}t.size!==0&&this.checkQueueFavorites(e,[...t])}async checkQueueFavorites(e,t){let n=await e.favoritesAmong(t);if(!n.success)return;let r=new Set(n.value??[]);for(let e of t)this.favoriteToggledShowIds.has(e)||r.has(e)&&this.setQueueFavorite(e,!0)}async openLoginModal(e){let t=this.appServices;if(t)try{let[{showLoginModal:n},r]=await Promise.all([_(()=>v(()=>import(`./show-login-modal-C9u5aoLY.js`).then(e=>e.n),__vite__mapDeps([25,2,5,26,3,27,1,4,6,7,28,29,30,31,32,33,34,35,36,37,38]))),t.get()]);n(await r.modalManager.get(),t,e)}catch{}}async handleSignedIn(){let e=this.appServices;if(e)try{await this.resolveFavoritesCollection(await e.get())}catch{}for(let e of this.favoriteToggledShowIds)this.favoriteSavingShowIds.has(e)||this.favoriteToggledShowIds.delete(e);this.favoriteCheckedShowIds.clear(),this.syncQueueFavorites()}async resolveFavoritesCollection(e){try{let t=await(await e.userService.get()).getLoggedInUser(),n=t.success?.itemname,r=t.error?.type===ie.userNotLoggedIn;n?this.favoritesCollection=`fav-${n.replace(/^@/,``)}`:r?this.favoritesCollection=null:this.favoritesCollection=P}catch{this.favoritesCollection=P}}setQueueFavorite(e,t){if(this.queueFavoriteShowIds.has(e)===t)return;let n=new Set(this.queueFavoriteShowIds);t?n.add(e):n.delete(e),this.queueFavoriteShowIds=n}advanceToQueuedTrack(e,t){if(e.outcome!==`advanced`){this.playbackEngine?.cancelPlayIntent();return}this.loadQueuedTrack(e.entry,t)}loadQueuedTrack(e,t){let n=this.ensurePlaybackEngine();this.displayTrackInDock(e.show,e.track),this.loadedEngineTrack=e.track,this.dockError=void 0,t&&(n.playWhenLoaded(),this.dockBuffering=!0),n.load(e.track),this.commitQueueChange()}createPlaybackEngine(){return new ht}handleShowPlayRequested(e){this.ensurePlaybackEngine().playWhenLoaded(),this.loadedEngineTrack=void 0,this.displayTrackInDock(e.detail.show,void 0),this.dockError=void 0;let{show:t,trackIndex:n,trackFileName:r}=e.detail;this.startShowPlayback(t,n,r)}handleShowQueueRequested(e){this.playRequestPending||this.appendShowToQueue(e.detail.show,e.detail.trackIndex)}handleDockPlayPauseToggled(){let e=this.ensurePlaybackEngine();if(e.isPlaying){e.pause();return}e.play();let t=this.dockTrack;!e.hasTrack&&t&&(e.load(t),this.loadedEngineTrack=t)}handleDockSeekRequested(e){this.playbackEngine?.seekTo(e.detail.positionSecs)}handleDockVolumeRequested(e){this.volumeLevel=et(e.detail.level),this.volumeChosen=!0,this.playbackEngine&&(this.playbackEngine.volume=this.volumeLevel),this.scheduleVolumePersist()}scheduleVolumePersist(){this.clearVolumePersistTimer(),this.volumePersistTimer=window.setTimeout(()=>{this.volumePersistTimer=void 0,this.persistVolume()},It)}flushVolumePersist(){this.volumePersistTimer!==void 0&&(this.clearVolumePersistTimer(),this.persistVolume())}clearVolumePersistTimer(){this.volumePersistTimer!==void 0&&(window.clearTimeout(this.volumePersistTimer),this.volumePersistTimer=void 0)}persistVolume(){this.userState?.saveVolume(this.volumeLevel)}async startShowPlayback(e,t,n){let r=this.showDetails;if(!r){this.playbackEngine?.cancelPlayIntent();return}let i=++this.dockRequestGeneration;this.playRequestPending=!0;try{let a=await r.getShowDetails(e.identifier);if(i!==this.dockRequestGeneration)return;this.playRequestPending=!1;let o=nt(a.setlist,t,n),s=n===void 0&&t!==void 0?a.setlist[t]:void 0,c=s!==void 0&&t!==void 0?this.playQueue.startTrack(e,s,t):this.playQueue.startShow(e,a.setlist,n===void 0?0:o);if(!c){this.concludeFailedPlayRequest(e);return}this.displayTrackInDock(c.show,c.track),this.dockError=void 0,this.loadedEngineTrack?.url!==c.track.url&&(this.dockBuffering=!0),this.ensurePlaybackEngine().load(c.track),this.loadedEngineTrack=c.track,this.commitQueueChange()}catch{if(i!==this.dockRequestGeneration)return;this.playRequestPending=!1,this.concludeFailedPlayRequest(e)}}concludeFailedPlayRequest(e){this.playbackEngine?.cancelPlayIntent(),this.dockError={kind:`show`,name:e.title};let t=this.playQueue.current;t?this.displayTrackInDock(t.show,t.track):this.readyFirstQueuedEntry(),this.refreshDockQueueState()}async appendShowToQueue(e,t){let n=this.showDetails;if(!n||t!==void 0&&(!Number.isInteger(t)||t<0||this.playQueue.hasTrack(e.identifier,t)))return;let r;try{r=await n.getShowDetails(e.identifier)}catch{return}let i=t===void 0?void 0:r.setlist[t];(t===void 0?this.playQueue.appendShow(e,r.setlist):i!==void 0&&this.playQueue.appendTrack(e,i,t))&&(!this.playRequestPending&&!this.playbackEngine?.hasTrack&&!this.playQueue.current&&(this.dockError=void 0,this.readyFirstQueuedEntry()),this.commitQueueChange())}readyFirstQueuedEntry(){let e=this.playQueue.startAtFirst();e&&(this.displayTrackInDock(e.show,e.track),this.ensurePlaybackEngine().load(e.track),this.loadedEngineTrack=e.track)}displayTrackInDock(e,t){this.dockShow=e,this.dockTrack=t;let n=t?.durationSecs??0;this.dockCurrentTimeSecs=0,this.dockDurationSecs=n,this.dockDurationFixed=n>0}refreshDockQueueState(){this.dockTrackCount=this.playQueue.trackCount,this.dockShowCount=this.playQueue.showCount,this.dockCanPrevious=this.playQueue.hasPrevious||this.recentlyPlayed.hasPullable(this.isQueued),this.dockRecentlyPlayed=this.recentlyPlayed.entries,this.dockCanNext=this.playQueue.hasNext,this.dockPlayOrder=this.playQueue.playOrder,this.syncQueueFavorites()}commitQueueChange(){this.queueTouched=!0,this.refreshDockQueueState(),this.scheduleQueuePersist()}scheduleQueuePersist(){this.clearQueuePersistTimer(),this.queuePersistTimer=window.setTimeout(()=>{this.queuePersistTimer=void 0,this.persistQueue()},Lt)}flushQueuePersist(){this.queuePersistTimer!==void 0&&(this.clearQueuePersistTimer(),this.persistQueue())}clearQueuePersistTimer(){this.queuePersistTimer!==void 0&&(window.clearTimeout(this.queuePersistTimer),this.queuePersistTimer=void 0)}persistQueue(){let e=this.userState;if(!e)return;let t=ot(this.playQueue.playOrder);t?e.saveQueue(t):e.clearQueue(),this.persistRecentlyPlayed(e)}persistRecentlyPlayed(e){if(!(this.recentlyPlayedDirty&&this.recentlyPlayedRestoreSettled))return;this.recentlyPlayedDirty=!1;let t=lt(this.recentlyPlayed.snapshot);e.saveRecentlyPlayed(t)}async restorePersistedHistory(e){let t=await e.loadRecentlyPlayed();t.status!==`failed`&&(t.status===`loaded`&&(this.recentlyPlayed.restore(ut(t.history,this.imageBaseUrl)),this.dockRecentlyPlayed=this.recentlyPlayed.entries),this.recentlyPlayedRestoreSettled=!0,this.recentlyPlayedDirty&&this.scheduleQueuePersist())}get queueRestorable(){return!this.queueTouched&&this.playQueue.trackCount===0&&!this.playRequestPending&&this.dockShow===void 0}async restorePersistedQueue(e,t){let n=await e.loadQueue();if(!n||!this.queueRestorable)return;let r=[...new Set(n.tracks.map(e=>e.showIdentifier))],i=await Promise.allSettled(r.map(e=>t.getShowDetails(e))),a=new Map;if(r.forEach((e,t)=>{let n=i[t];n?.status===`fulfilled`&&a.set(e,n.value)}),!this.queueRestorable)return;let o=ct(n,a,this.imageBaseUrl);if(!o)return;let s=this.playQueue.restore(o.entries,o.currentIndex);s&&(this.displayTrackInDock(s.show,s.track),this.refreshDockQueueState())}handleArtistSelected(e){this.navigateToUrl(L(e.detail.artist.identifier))}navigateToUrl(e){window.location.href=e}handleResize(e){let t=this.appRootController?.getWindowScrollbarWidth()??0,n=e.contentRect.width+t;this.mode=rt(n),this.modeReady=!0}async setupAppServicesDependents(){if(this.setupStarted)return;let e=this.appServices;if(e){this.setupStarted=!0;try{let t=await e.get();this.resolveFavoritesCollection(t),await Promise.allSettled([this.setupDataServices(t),this.setupPageMetadata(t),this.setupResizeObserver(t)])}catch{}finally{this.resizeObserver||(this.modeReady=!0),this.servicesReady=!0}}}async setupDataServices(e){let[t,n,r,i,a]=await Promise.all([e.searchService.get(),e.appConfig.get(),e.favoriteService.get(),e.keyValueStoreManager.get(),e.metadataService.get()]);this.imageBaseUrl=n.baseImageUrl,this.searchAdapter=new qe(t,{imageBaseUrl:n.baseImageUrl});let o=new $e({favoriteService:r,keyValueStoreManager:i});this.userState=o,this.artistCounts=new Ce(t);let s=new Qe(a);this.showDetails=s,this.artistAbout=new Se(()=>e.wikipediaAPIService.get()),this.restorePersistedQueue(o,s),this.restorePersistedHistory(o),await Promise.all([this.applyPersistedSectionOrder(o),this.applyPersistedVolume(o)])}async applyPersistedSectionOrder(e){let t=await e.loadSectionOrder();t!==void 0&&this.sectionOrder===O&&(this.sectionOrder=bt(t,O))}async applyPersistedVolume(e){if(this.volumeChosen){this.persistVolume();return}let t=await e.loadVolume();if(this.volumeChosen){this.persistVolume();return}t!==void 0&&(this.volumeLevel=t,this.playbackEngine&&(this.playbackEngine.volume=t))}async setupPageMetadata(e){(await e.pageMetadataService.get()).setTitle(o(`Live Music Archive`))}async setupResizeObserver(e){let[t,n]=await Promise.allSettled([e.appRootController.get(),e.resizeObserver.get()]);t.status===`fulfilled`&&(this.appRootController=t.value),n.status===`fulfilled`&&(this.resizeObserver=n.value,this.observeSceneRoot())}observeSceneRoot(){let e=this.resizeObserver,t=this.sceneRoot;!e||!t||!this.isConnected||e.addObserver({handler:this,target:t})}disconnectResizeObserver(){let e=this.sceneRoot;e&&this.resizeObserver?.removeObserver({handler:this,target:e})}static get styles(){return[R,e`
        :host {
          display: block;
        }

        #scene-root {
          /* LMA brand chrome: identity colors, not theme colors, defined once
             here and inherited into every LMA component's shadow root. The
             player accent is the player's fixed lime in every theme, and the
             progress fill the player's fixed indigo. */
          --lma-purple-gradient: linear-gradient(
            120deg,
            #4c1d95,
            #6d28d9 55%,
            #7c3aed
          );
          --lma-rainbow-gradient: linear-gradient(
            90deg,
            #f59e0b,
            #ef4444,
            #ec4899,
            #8b5cf6,
            #3b82f6,
            #14b8a6
          );
          /* The LMA History hero's purple: the canonical gradient with its
             stops shifted one shade deeper. */
          --lma-history-purple-gradient: linear-gradient(
            120deg,
            #312e81,
            #4c1d95 55%,
            #6d28d9
          );
          /* The content surfaces' violet and indigo action buttons (About
             Tapers and the History cross-link) and the violet glow. */
          --lma-violet-button-gradient: linear-gradient(
            135deg,
            #7c3aed,
            #6d28d9
          );
          --lma-violet-button-glow: rgba(124, 58, 237, 0.4);
          --lma-indigo-button-gradient: linear-gradient(
            135deg,
            #4c1d95,
            #312e81
          );
          /* Inks on the purple chrome: white text, its soft violet body
             tint, the amber subtitle highlight, and the amber hero icon. */
          --lma-chrome-ink: #fff;
          --lma-chrome-ink-soft: #ede9fe;
          --lma-chrome-highlight: #fde68a;
          --lma-chrome-icon: #fcd34d;
          /* The hero links' indigo and violet inks with their hover shades,
             and the violet accents the Taper Section's cards take on
             hover. */
          --lma-indigo-ink: #4338ca;
          --lma-indigo-ink-hover: #312e81;
          --lma-violet-ink: #6d28d9;
          --lma-violet-ink-hover: #4c1d95;
          --lma-violet-accent: #a78bfa;
          --lma-violet-accent-strong: #8b5cf6;
          /* The Upload Recording link's blue-to-indigo fill and its hover
             glow. */
          --lma-upload-gradient: linear-gradient(135deg, #2563eb, #4f46e5);
          --lma-upload-glow: rgba(79, 70, 229, 0.45);
          --lma-player-accent: #aaff00;
          --lma-player-accent-glow: rgba(170, 255, 0, 0.67);
          --lma-player-progress-fill: rgb(99, 102, 241);
          /* The expanded panel's pressed Queue view control: a green wash
             under green ink. The queue list's current row: bold blue ink. */
          --lma-player-panel-queue-active-bg: rgba(34, 197, 94, 0.15);
          --lma-player-panel-queue-active-ink: #16a34a;
          --lma-player-current-row-ink: #2563eb;
          /* The player bar is a fixed light surface by design: the
             prototype's dark variants are stripped per the scene README, so
             the surface and the ink drawn on it stay the same in every
             theme. Surface: its body, the strip gradient behind its
             controls, and its border, plus the plain translucent strip and
             top rule of the idle mobile bar. */
          --lma-player-bg: linear-gradient(135deg, #fef3e8 0%, #e8f4f8 100%);
          --lma-player-strip-bg: linear-gradient(
            135deg,
            #fce8e6 0%,
            #d4e8f4 100%
          );
          --lma-player-border: rgba(0, 0, 0, 0.08);
          --lma-player-idle-bg: rgba(255, 255, 255, 0.95);
          --lma-player-idle-border: #e5e5e5;
          /* Player ink: text, control glyphs, dividers, hover wash, the play
             button, the error line, and the focus ring. */
          --lma-player-ink: #171717;
          --lma-player-ink-secondary: #525252;
          --lma-player-ink-muted: #6b6b6b;
          --lma-player-control-ink: #404040;
          --lma-player-idle-control-ink: #737373;
          --lma-player-faint-ink: #a3a3a3;
          --lma-player-road-trip-ink: #f59e0b;
          --lma-player-divider: rgba(163, 163, 163, 0.3);
          --lma-player-control-hover-bg: rgba(255, 255, 255, 0.5);
          --lma-player-play-bg: linear-gradient(
            to bottom right,
            rgba(59, 130, 246, 0.2),
            rgba(168, 85, 247, 0.2)
          );
          --lma-player-play-hover-bg: linear-gradient(
            to bottom right,
            rgba(59, 130, 246, 0.3),
            rgba(168, 85, 247, 0.3)
          );
          --lma-player-play-border: rgba(96, 165, 250, 0.3);
          --lma-player-play-ink: #2563eb;
          --lma-player-idle-play-border: #d4d4d4;
          --lma-player-danger-ink: #b91c1c;
          --lma-player-focus-ring: #2563eb;
          /* The scene uses the prototype's system font stack in place of the
             site's base font; components inside the scene, the player dock
             included, read it through this variable or inherit it. */
          --ia-theme-base-font-family:
            ui-sans-serif, system-ui, sans-serif, 'Apple Color Emoji',
            'Segoe UI Emoji', 'Segoe UI Symbol', 'Noto Color Emoji';
          /* The player queue: the bar's queue toggle's orange wash (with a
             ring on mobile), the tray's white surface and violet-to-pink
             header wash, the list's purple show headers, and its
             blue-washed current row. Like the bar, the queue is a fixed
             light surface in every theme. */
          --lma-player-queue-toggle-active-bg: rgba(249, 115, 22, 0.3);
          --lma-player-queue-toggle-active-ring: rgba(249, 115, 22, 0.6);
          --lma-player-queue-tray-bg: linear-gradient(
            135deg,
            #ffffff 0%,
            #f8f9fa 100%
          );
          --lma-player-queue-tray-border: rgba(0, 0, 0, 0.1);
          --lma-player-queue-tray-header-bg: linear-gradient(
            135deg,
            rgba(139, 92, 246, 0.05) 0%,
            rgba(236, 72, 153, 0.05) 100%
          );
          --lma-player-queue-tray-header-border: rgba(0, 0, 0, 0.06);
          --lma-player-queue-tray-close-hover-bg: rgba(0, 0, 0, 0.1);
          --lma-player-queue-header-bg: rgba(250, 245, 255, 0.9);
          --lma-player-queue-header-hover-bg: rgba(243, 232, 255, 0.9);
          --lma-player-queue-header-border: rgba(233, 213, 255, 0.6);
          --lma-player-queue-header-glyph: #c084fc;
          --lma-player-queue-header-title: #581c87;
          --lma-player-queue-header-title-playing: #7e22ce;
          --lma-player-queue-header-meta: rgba(168, 85, 247, 0.9);
          --lma-player-queue-header-action-hover-bg: #f3e8ff;
          --lma-player-queue-header-action-hover-ink: #7e22ce;
          --lma-player-queue-favorite-ink: #ef4444;
          --lma-player-queue-remove-hover-ink: #ef4444;
          --lma-player-queue-row-border: rgba(0, 0, 0, 0.05);
          --lma-player-queue-row-hover-bg: #fafafa;
          --lma-player-queue-current-row-bg: #eff6ff;
          --lma-player-queue-action-hover-ink: #3b82f6;
          background: var(--ia-theme-primary-background-color, #fbfbfd);
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          font-family: var(--ia-theme-base-font-family, inherit);
        }

        #page-container {
          max-width: var(--app-page-max-width, 135rem);
          margin: 0 auto;
          padding: 4px 32px 40px;
        }

        #scene-root.mobile #page-container {
          padding: 4px 16px 32px;
        }

        /*
         * The links row: right-aligned on one line, except in mobile mode,
         * where the two buttons take the first line (right-aligned) and the
         * two links the second, spread to its edges.
         */
        .hero-links {
          display: flex;
          flex-wrap: wrap;
          align-items: center;
          justify-content: flex-end;
          gap: 12px;
          padding-top: 20px;
        }

        /* In tablet and desktop modes the row reaches 12px into the page
           container's 32px side padding, so its content sits 20px from the
           container's edges; in mobile mode the container's 16px padding
           sets the inset. */
        #scene-root.tablet .hero-links,
        #scene-root.desktop .hero-links {
          margin-inline: -12px;
        }

        .hero-link-group {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        #scene-root.mobile .hero-links {
          flex-direction: column;
          align-items: stretch;
          gap: 8px;
        }

        #scene-root.mobile .hero-link-group {
          justify-content: flex-end;
          gap: 8px;
        }

        #scene-root.mobile .hero-link-group.actions {
          justify-content: space-between;
        }

        .hero-link {
          padding: 6px 8px;
          border: 0;
          background: transparent;
          font-family: inherit;
          font-size: 14px;
          line-height: 20px;
          font-weight: 600;
          text-underline-offset: 4px;
          cursor: pointer;
          transition: color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
        }

        .hero-link:hover {
          text-decoration: underline;
        }

        .history-link {
          color: var(--lma-indigo-ink);
        }

        .history-link:hover {
          color: var(--lma-indigo-ink-hover);
        }

        .tapers-link {
          color: var(--lma-violet-ink);
        }

        .tapers-link:hover {
          color: var(--lma-violet-ink-hover);
        }

        .classic-mode-link,
        .upload-link {
          display: flex;
          align-items: center;
          padding: 6px 12px;
          border-radius: 8px;
          font-size: 14px;
          line-height: 20px;
          font-weight: 600;
          text-decoration: none;
        }

        .classic-mode-link {
          border: 1px solid var(--ia-theme-primary-cta-border, #c5d1df);
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          transition: background-color 0.15s cubic-bezier(0.4, 0, 0.2, 1);
        }

        .classic-mode-link:hover {
          background: var(--ia-theme-primary-background-color, #fbfbfd);
        }

        .upload-link {
          gap: 8px;
          background: var(--lma-upload-gradient);
          color: var(--lma-chrome-ink);
          box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);
          transition:
            transform 0.15s cubic-bezier(0.4, 0, 0.2, 1),
            box-shadow 0.15s cubic-bezier(0.4, 0, 0.2, 1);
        }

        .upload-link:hover {
          transform: scale(1.05);
          box-shadow: 0 0 20px var(--lma-upload-glow);
        }

        .hero-link:focus-visible,
        .classic-mode-link:focus-visible,
        .upload-link:focus-visible {
          outline: 2px solid var(--ia-theme-link-color, #4b64ff);
          outline-offset: 2px;
        }

        @media (prefers-reduced-motion: reduce) {
          .hero-link,
          .classic-mode-link,
          .upload-link {
            transition: none;
          }

          .upload-link:hover {
            transform: none;
          }
        }

        /*
         * One breakpoint, like the reference build: desktop keeps the hero
         * and the artwork entry side by side on a single row (the artwork
         * scales down at narrow desktop widths rather than wrapping into a
         * mostly-empty second row), and below desktop the hero stacks with
         * the compact entry row beneath the search bar while the Concert
         * Browser is available.
         */
        .hero {
          display: flex;
          align-items: flex-end;
          gap: 24px;
          padding: 12px 0 16px;
        }

        #scene-root.tablet .hero,
        #scene-root.mobile .hero {
          flex-direction: column;
          align-items: stretch;
          gap: 16px;
        }

        #scene-root.desktop .hero {
          gap: 50px;
        }

        .hero-main {
          min-width: 0;
        }

        .hero-title-row {
          display: flex;
          align-items: flex-end;
          gap: 16px;
          margin-bottom: 4px;
        }

        /*
         * The logo image is square, so the explicit width matching the height
         * reserves its box before the image loads and the wordmark paints in
         * its final position.
         */
        .hero-logo {
          height: 120px;
          width: 120px;
          flex-shrink: 0;
        }

        #scene-root.mobile .hero-logo {
          height: 64px;
          width: 64px;
        }

        .hero-heading {
          margin: 0;
          font-size: 36px;
          line-height: 1.15;
          font-weight: 700;
          white-space: nowrap;
        }

        /*
         * The wordmark wraps in mobile mode so the title row fits viewports
         * narrower than the logo plus the one-line wordmark (about 342px);
         * each wrapped line carries an enlarged initial, so the unitless
         * line-height keeps the line boxes evenly sized.
         */
        #scene-root.mobile .hero-heading {
          font-size: 24px;
          white-space: normal;
        }

        .hero-heading .initial {
          font-size: 1.6em;
        }

        .hero-tagline {
          margin: 0 0 16px;
          font-family: 'Helvetica Neue', sans-serif;
          font-size: 24px;
          font-weight: 500;
          line-height: 1.3;
          letter-spacing: -1.2px;
          color: var(--ia-theme-secondary-text-color, #666);
        }

        #scene-root.tablet .hero-tagline {
          font-size: 20px;
        }

        #scene-root.mobile .hero-tagline {
          font-size: 17px;
        }

        /*
         * The search field is deliberately black-on-white in every theme.
         * The clearable-input component's custom properties recreate the
         * field treatment: 50px overall height (24px line + 12px vertical
         * padding + 1px border each side), the black hairline border, and
         * the 8px rounding. The clear-button variables center the button's
         * 32px hit area in the 50px field. The component's own default
         * focus glow is disabled so the host focus ring below is the only
         * focus treatment.
         */
        /*
         * The component's shadow input uses content-box sizing (the app's
         * border-box reset does not cross its shadow boundary), so its
         * rendered box is input-height + vertical padding + borders. Vertical
         * padding stays 0 and the height carries the full content size so the
         * box lands at 48 + 2px of borders = 50px, matching the previous
         * field; single-line inputs center their text vertically on their
         * own. The focus ring is drawn by the input itself via the focused
         * box-shadow variable so it hugs the real rendered box (an outline on
         * this host would trace the smaller host box instead).
         */
        .hero-search {
          display: block;
          width: 100%;
          max-width: 690px;
          --input-height: 48px;
          --input-padding: 0 16px;
          --input-border-color: #000;
          --input-border-radius: 8px;
          --input-background-color: #fff;
          --input-color: #000;
          --input-font-size: 16px;
          --input-line-height: 24px;
          --input-box-shadow: none;
          --input-focused-border-color: #000;
          --input-focused-box-shadow: 0 0 0 2px
            var(--ia-theme-link-color, #4b64ff);
          --clear-button-top: 8px;
          --clear-button-right: 9px;
          --clear-button-height: 32px;
          --clear-button-width: 32px;
        }

        .concert-browser-entry {
          display: flex;
          flex-direction: column;
          align-items: flex-end;
          gap: 8px;
          flex-shrink: 0;
          margin-left: auto;
        }

        .concert-browser-entry.compact {
          flex-direction: row;
          align-items: center;
          margin-left: 0;
        }

        .concert-browser-open {
          padding: 8px 16px;
          border: 1px solid var(--ia-theme-primary-cta-border, #c5d1df);
          border-radius: 8px;
          background: var(--ia-theme-secondary-background-color, #fff);
          color: var(--ia-theme-primary-text-color, #2c2c2c);
          font-size: var(--ia-theme-font-size-standard, 1.4rem);
          font-weight: 600;
          cursor: pointer;
        }

        .concert-browser-title {
          font-size: 24px;
          line-height: 32px;
          font-weight: 600;
          letter-spacing: -0.025em;
        }

        /* The artwork carries its own near-white background, a shade lighter
           than the hero's, so darken lets the hero colour win wherever the
           artwork is lighter and its edges merge into the page. */
        .concert-browser-image {
          width: clamp(220px, 30vw, 360px);
          aspect-ratio: 1 / 1;
          height: auto;
          max-width: 100%;
          object-fit: contain;
          border-radius: 8px;
          mix-blend-mode: darken;
        }

        /* Forced colours paint the hero in the system canvas colour, which a
           dark theme makes black; the unblended artwork stays visible there. */
        @media (forced-colors: active) {
          .concert-browser-image {
            mix-blend-mode: normal;
          }
        }

        #sections {
          display: flex;
          flex-direction: column;
          gap: 28px;
          margin-top: 16px;
        }

        .section-slot {
          position: relative;
        }

        .section-slot.drag-source {
          opacity: 0.6;
        }

        /*
         * The drop-line indicator marking where a dragged section would land:
         * a thin accent line spanning the section column, sitting in the 28px
         * gap between sections (16px clears the wrapper's edge and centers
         * the 3px line, dot included, in the gap). pointer-events: none keeps
         * the line from stealing the dragover events that position it, and
         * the drag-indicator layer keeps it above the sections' own content.
         */
        .drop-line {
          position: absolute;
          left: 0;
          right: 0;
          height: 3px;
          border-radius: 2px;
          background: var(--ia-theme-link-color, #4b64ff);
          box-shadow: 0 0 8px var(--ia-theme-link-color, #4b64ff);
          pointer-events: none;
          z-index: var(--lma-z-drag-indicator);
        }

        .drop-line.before {
          top: -16px;
        }

        .drop-line.after {
          bottom: -16px;
        }

        /* An 11px ring centered on the line's left end, capping it. */
        .drop-line-dot {
          position: absolute;
          left: -5px;
          top: -4px;
          box-sizing: border-box;
          width: 11px;
          height: 11px;
          border: 3px solid var(--ia-theme-link-color, #4b64ff);
          border-radius: 50%;
          background: var(--ia-theme-primary-background-color, #fbfbfd);
        }
      `]}};y([t({type:Object})],$.prototype,`appServices`,void 0),y([t({type:Object})],$.prototype,`capabilities`,void 0),y([s()],$.prototype,`showMoreSection`,void 0),y([s()],$.prototype,`infoSurface`,void 0),y([g(`lma-overlay.info-overlay`)],$.prototype,`infoOverlay`,void 0),y([s()],$.prototype,`tapingStats`,void 0),y([g(`lma-player-dock`)],$.prototype,`playerDock`,void 0),y([g(`lma-floater-host`)],$.prototype,`floaterHost`,void 0),y([g(`lma-show-more-surface`)],$.prototype,`showMoreSurface`,void 0),y([s()],$.prototype,`mode`,void 0),y([s()],$.prototype,`modeReady`,void 0),y([s()],$.prototype,`searchAdapter`,void 0),y([s()],$.prototype,`userState`,void 0),y([s()],$.prototype,`artistCounts`,void 0),y([s()],$.prototype,`showDetails`,void 0),y([s()],$.prototype,`servicesReady`,void 0),y([s()],$.prototype,`heroSearchQuery`,void 0),y([s()],$.prototype,`dockShow`,void 0),y([s()],$.prototype,`dockTrack`,void 0),y([s()],$.prototype,`dockPlaying`,void 0),y([s()],$.prototype,`dockBuffering`,void 0),y([s()],$.prototype,`dockError`,void 0),y([s()],$.prototype,`dockCurrentTimeSecs`,void 0),y([s()],$.prototype,`dockDurationSecs`,void 0),y([s()],$.prototype,`playRequestPending`,void 0),y([s()],$.prototype,`dockRecentlyPlayed`,void 0),y([s()],$.prototype,`dockTrackCount`,void 0),y([s()],$.prototype,`dockShowCount`,void 0),y([s()],$.prototype,`dockCanPrevious`,void 0),y([s()],$.prototype,`dockCanNext`,void 0),y([s()],$.prototype,`dockPlayOrder`,void 0),y([s()],$.prototype,`favoritesCollection`,void 0),y([s()],$.prototype,`sessionFavoritedShows`,void 0),y([s()],$.prototype,`sessionUnfavoritedShowIds`,void 0),y([s()],$.prototype,`queueFavoriteShowIds`,void 0),y([s()],$.prototype,`volumeLevel`,void 0),y([s()],$.prototype,`sectionOrder`,void 0),y([s()],$.prototype,`armedSectionId`,void 0),y([s()],$.prototype,`dragSectionId`,void 0),y([s()],$.prototype,`dropTarget`,void 0),y([g(`#scene-root`)],$.prototype,`sceneRoot`,void 0),$=y([h(`live-music-archive-page`),i()],$);var Bt=$;export{Bt as default};