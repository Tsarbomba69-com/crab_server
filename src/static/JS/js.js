// abre e fecha o menu
const nav = document.querySelector('#cabecalho nav')
const toggle = document.querySelectorAll('nav .toggle')
for(const element of toggle){
    element.addEventListener('click', function(){
        nav.classList.toggle('mostra')
    })
}
const links=document.querySelectorAll('nav ul li a')
for (const link of links){
    link.addEventListener('click', function(){
        nav.classList.remove('mostra')
    })
}

// selecionar menu quando rolar a pagina



// rolagem da pagina
// window.addEventListener('scroll',function(){
//     changeHeaderWhenScroll()
//     backToTop()
//     activarMenu()
// })
