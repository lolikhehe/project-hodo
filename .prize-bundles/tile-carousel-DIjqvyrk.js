import{A as e,C as t,E as n,O as r,T as i,p as a,w as o}from"./vendor-lit-MQ4DzmMc.js";import{t as s}from"./index-NFRVgqV2.js";import{a as c,i as l}from"./dist-P4PnKJsG.js";async function u(e,t){let n=t??l.default,r={query:e},i=await n.search(r,c.FEDERATED);if(i.success)return i.success.response.federatedResults;console.error(i.error)}var d=class extends i{constructor(...e){super(...e),this.usePlaceholder=!1,this.tileType=`grid`,this.layoutType=`default`,this.enableHoverPanes=!1,this.numPlaceholders=7,this.baseImageUrl=`https://archive.org`,this.baseNavigationUrl=``}render(){return this.usePlaceholder?this.placeholderTilesTemplate:this.tilesTemplate}get tilesTemplate(){return this.tiles?r`${a(this.uniqueTiles,e=>e.identifier,e=>r`
          <tile-dispatcher
            .model=${e}
            .baseImageUrl=${this.baseImageUrl}
            .baseNavigationUrl=${this.baseNavigationUrl}
            .tileDisplayMode=${this.tileType}
            .layoutType=${this.layoutType}
            .enableHoverPane=${this.enableHoverPanes}
            @resultSelected=${this.tileClicked}
          ></tile-dispatcher>
        `)}`:n}get uniqueTiles(){let e=new Set;return(this.tiles??[]).filter(t=>{let{identifier:n}=t;return!n||e.has(n)?!1:(e.add(n),!0)})}get placeholderTilesTemplate(){return r`${Array.from({length:this.numPlaceholders},()=>r`<collection-browser-loading-tile></collection-browser-loading-tile>`)}`}tileClicked(e){this.dispatchEvent(new CustomEvent(`tileClick`,{detail:e.detail}))}static get styles(){return e`
      :host {
        margin: 0 1px 2px;
        padding: 10px 10px 8px;
        display: flex;
        flex-direction: row;
        gap: 1.7rem;
        overflow-x: auto;
        scrollbar-width: thin;

        --sixTileWidth: calc(17% - 1.8rem);
        --fiveTileWidth: calc(20% - 1.4rem);
        --fourTileWidth: calc(25% - 1.3rem);
        --threeTileWidth: calc(33% - 0.9rem);
        --twoTileWidth: calc(50% - 0.85rem);
        --oneTileWidth: 100%;

        --sevenTileWidthWide: calc(14.2857% - 1.514rem);
        --sixTileWidthWide: calc(16.6667% - 1.483rem);
        --fiveTileWidthWide: calc(20% - 1.44rem);
        --fourTileWidthWide: calc(25% - 1.375rem);
        --threeTileWidthWide: calc(33.3333% - 1.226rem);
        --twoTileWidthWide: calc(50% - 0.85rem);
        --oneTileWidthWide: 100%;
      }

      tile-dispatcher {
        flex: 0 0 var(--sixTileWidth, 16%);
      }

      collection-browser-loading-tile {
        height: 20rem;
      }

      @supports (display: grid) {
        :host {
          display: grid;
          row-gap: 1.7rem;
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--sixTileWidth, 16%), 1fr)
          );
          grid-auto-flow: column;
          grid-auto-columns: minmax(var(--sixTileWidth, 16%), 1fr);
        }

        :host(.grid) {
          grid-auto-flow: unset;
        }

        :host(.wide) {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--sevenTileWidthWide, 14%), 1fr)
          );
          grid-auto-columns: minmax(var(--sevenTileWidthWide, 14%), 1fr);
        }
      }

      /** Adjust size to match infinite scroller grid at breakpoints */
      @media only screen and (max-width: 1342px) {
        :host(.wide) {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--sixTileWidthWide, 16%), 1fr)
          );
          grid-auto-columns: minmax(var(--sixTileWidthWide, 16%), 1fr);
        }

        :host(.wide) tile-dispatcher {
          flex: 0 0 var(--sixTileWidthWide, 16%);
        }
      }

      @media only screen and (max-width: 1361px) {
        :host {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--fiveTileWidth, 19%), 1fr)
          );
          grid-auto-columns: minmax(var(--fiveTileWidth, 19%), 1fr);
        }

        tile-dispatcher {
          flex: 0 0 var(--fiveTileWidth, 19%);
        }
      }

      @media only screen and (max-width: 1155px) {
        :host(.wide) {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--fiveTileWidthWide, 20%), 1fr)
          );
          grid-auto-columns: minmax(var(--fiveTileWidthWide, 20%), 1fr);
        }

        :host(.wide) tile-dispatcher {
          flex: 0 0 var(--fiveTileWidthWide, 20%);
        }
      }

      @media only screen and (max-width: 1174px) {
        :host {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--fourTileWidth, 24%), 1fr)
          );
          grid-auto-columns: minmax(var(--fourTileWidth, 24%), 1fr);
        }

        tile-dispatcher {
          flex: 0 0 var(--fourTileWidth, 24%);
        }
      }

      @media only screen and (max-width: 968px) {
        :host(.wide) {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--fourTileWidthWide, 25%), 1fr)
          );
          grid-auto-columns: minmax(var(--fourTileWidthWide, 25%), 1fr);
        }

        :host(.wide) tile-dispatcher {
          flex: 0 0 var(--fourTileWidthWide, 25%);
        }
      }

      @media only screen and (max-width: 987px) {
        :host {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--threeTileWidth, 32%), 1fr)
          );
          grid-auto-columns: minmax(var(--threeTileWidth, 32%), 1fr);
        }

        tile-dispatcher {
          flex: 0 0 var(--threeTileWidth, 32%);
        }
      }

      @media only screen and (max-width: 740px) {
        :host {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--twoTileWidth, 49%), 1fr)
          );
          grid-auto-columns: minmax(var(--twoTileWidth, 49%), 1fr);
        }

        tile-dispatcher {
          flex: 0 0 var(--twoTileWidth, 49%);
        }
      }

      @media only screen and (max-width: 701px) {
        :host(.wide) {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--threeTileWidthWide, 32%), 1fr)
          );
          grid-auto-columns: minmax(var(--threeTileWidthWide, 32%), 1fr);
        }

        :host(.wide) tile-dispatcher {
          flex: 0 0 var(--threeTileWidthWide, 32%);
        }
      }

      @media only screen and (max-width: 629px) {
        :host {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--threeTileWidth, 32%), 1fr)
          );
          grid-auto-columns: minmax(var(--threeTileWidth, 32%), 1fr);
        }

        tile-dispatcher {
          flex: 0 0 var(--threeTileWidth, 32%);
        }
      }

      @media only screen and (max-width: 509px) {
        :host(.wide) {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--twoTileWidthWide, 49%), 1fr)
          );
          grid-auto-columns: minmax(var(--twoTileWidthWide, 49%), 1fr);
        }

        :host(.wide) tile-dispatcher {
          flex: 0 0 var(--twoTileWidthWide, 49%);
        }
      }

      @media only screen and (max-width: 524px) {
        :host {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--twoTileWidth, 49%), 1fr)
          );
          grid-auto-columns: minmax(var(--twoTileWidth, 49%), 1fr);
        }

        tile-dispatcher {
          flex: 0 0 var(--twoTileWidth, 49%);
        }
      }

      @media only screen and (max-width: 282px) {
        :host,
        :host(.wide) {
          grid-template-columns: repeat(
            auto-fill,
            minmax(var(--oneTileWidth, 100%), 1fr)
          );
          grid-auto-columns: minmax(var(--oneTileWidth, 100%), 1fr);
        }

        tile-dispatcher,
        :host(.wide) tile-dispatcher {
          flex: 0 0 var(--oneTileWidth, 100%);
        }
      }
    `}};s([t({type:Array})],d.prototype,`tiles`,void 0),s([t({type:Boolean})],d.prototype,`usePlaceholder`,void 0),s([t({type:String})],d.prototype,`tileType`,void 0),s([t({type:String})],d.prototype,`layoutType`,void 0),s([t({type:Boolean})],d.prototype,`enableHoverPanes`,void 0),s([t({type:Number})],d.prototype,`numPlaceholders`,void 0),s([t({type:String})],d.prototype,`baseImageUrl`,void 0),s([t({type:String})],d.prototype,`baseNavigationUrl`,void 0),d=s([o(`tile-carousel`)],d);export{u as t};