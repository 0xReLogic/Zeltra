
import jsPDF from 'jspdf'
import autoTable from 'jspdf-autotable'

export function exportPDF(
    title: string,
    headers: string[],
    data: (string | number)[][],
    filename: string
) {
    const doc = new jsPDF()

    // Add Title
    doc.setFontSize(18)
    doc.text(title, 14, 22)
    
    // Add Date
    doc.setFontSize(11)
    doc.setTextColor(100)
    doc.text(`Generated on: ${new Date().toLocaleDateString()}`, 14, 30)

    // Add Table
    autoTable(doc, {
        head: [headers],
        body: data,
        startY: 35,
        theme: 'grid',
        headStyles: { fillColor: [22, 163, 74] }, // Emerald-600 matches theme
        styles: { fontSize: 10, cellPadding: 3 },
    })

    doc.save(filename)
}
