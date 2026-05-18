declare module "react-cytoscapejs" {
  import cytoscape from "cytoscape"
  import * as React from "react"

  export interface CytoscapeComponentProps {
    elements?: any
    style?: React.CSSProperties
    className?: string
    layout?: any
    stylesheet?: any
    cy?: (cy: cytoscape.Core) => void
    [key: string]: any
  }

  const CytoscapeComponent: React.ComponentType<CytoscapeComponentProps>

  export default CytoscapeComponent
}